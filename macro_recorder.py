from pynput import mouse, keyboard
from pynput.mouse import Button, Controller as MouseController
from pynput.keyboard import Key, Controller as KeyboardController
import time
import threading

class MacroRecorder:
    def __init__(self):
        self.events = []
        self.recording = False
        self.start_time = None
        self.mouse_listener = None
        self.keyboard_listener = None
    
    def start_recording(self):
        """Inicia la grabación de eventos"""
        self.events = []
        self.recording = True
        self.start_time = time.time()
        
        # Listener para mouse
        self.mouse_listener = mouse.Listener(
            on_move=self.on_move,
            on_click=self.on_click,
            on_scroll=self.on_scroll
        )
        
        # Listener para teclado
        self.keyboard_listener = keyboard.Listener(
            on_press=self.on_press,
            on_release=self.on_release
        )
        
        self.mouse_listener.start()
        self.keyboard_listener.start()
    
    def stop_recording(self):
        """Detiene la grabación de eventos"""
        self.recording = False
        
        if self.mouse_listener:
            self.mouse_listener.stop()
        if self.keyboard_listener:
            self.keyboard_listener.stop()
        
        return self.events
    
    def on_move(self, x, y):
        """Registra movimiento del mouse (reducido para evitar sobrecarga)"""
        if self.recording:
            timestamp = time.time() - self.start_time
            # Reducir eventos de movimiento para mejor rendimiento
            if not self.events or timestamp - self.events[-1].get('timestamp', 0) > 0.05:
                self.events.append({
                    'type': 'mouse_move',
                    'x': x,
                    'y': y,
                    'timestamp': timestamp
                })
    
    def on_click(self, x, y, button, pressed):
        """Registra clicks del mouse"""
        if self.recording:
            timestamp = time.time() - self.start_time
            self.events.append({
                'type': 'mouse_click',
                'x': x,
                'y': y,
                'button': str(button),
                'pressed': pressed,
                'timestamp': timestamp
            })
    
    def on_scroll(self, x, y, dx, dy):
        """Registra scroll del mouse"""
        if self.recording:
            timestamp = time.time() - self.start_time
            self.events.append({
                'type': 'mouse_scroll',
                'x': x,
                'y': y,
                'dx': dx,
                'dy': dy,
                'timestamp': timestamp
            })
    
    def on_press(self, key):
        """Registra teclas presionadas"""
        if self.recording:
            timestamp = time.time() - self.start_time
            try:
                key_char = key.char
            except AttributeError:
                key_char = str(key)
            
            self.events.append({
                'type': 'key_press',
                'key': key_char,
                'timestamp': timestamp
            })
    
    def on_release(self, key):
        """Registra teclas soltadas"""
        if self.recording:
            timestamp = time.time() - self.start_time
            try:
                key_char = key.char
            except AttributeError:
                key_char = str(key)
            
            self.events.append({
                'type': 'key_release',
                'key': key_char,
                'timestamp': timestamp
            })


class MacroPlayer:
    def __init__(self):
        self.mouse = MouseController()
        self.keyboard_ctrl = KeyboardController()
        self.playing = False
        self.play_thread = None
    
    def play_macro(self, events, speed=1.0):
        """Reproduce una macro"""
        if self.playing:
            return False
        
        self.playing = True
        self.play_thread = threading.Thread(
            target=self._play_events,
            args=(events, speed),
            daemon=True
        )
        self.play_thread.start()
        return True
    
    def stop_playback(self):
        """Detiene la reproducción"""
        self.playing = False
    
    def _play_events(self, events, speed):
        """Reproduce los eventos grabados"""
        if not events:
            self.playing = False
            return
        
        start_time = time.time()
        last_timestamp = 0
        
        for event in events:
            if not self.playing:
                break
            
            # Esperar el tiempo correspondiente
            target_time = event['timestamp'] / speed
            elapsed = time.time() - start_time
            sleep_time = target_time - elapsed
            
            if sleep_time > 0:
                time.sleep(sleep_time)
            
            # Ejecutar el evento
            self._execute_event(event)
        
        self.playing = False
    
    def _execute_event(self, event):
        """Ejecuta un evento específico"""
        try:
            if event['type'] == 'mouse_move':
                self.mouse.position = (event['x'], event['y'])
            
            elif event['type'] == 'mouse_click':
                button = self._parse_button(event['button'])
                if event['pressed']:
                    self.mouse.press(button)
                else:
                    self.mouse.release(button)
            
            elif event['type'] == 'mouse_scroll':
                self.mouse.scroll(event['dx'], event['dy'])
            
            elif event['type'] == 'key_press':
                key = self._parse_key(event['key'])
                self.keyboard_ctrl.press(key)
            
            elif event['type'] == 'key_release':
                key = self._parse_key(event['key'])
                self.keyboard_ctrl.release(key)
        
        except Exception as e:
            print(f"Error ejecutando evento: {e}")
    
    def _parse_button(self, button_str):
        """Convierte string a Button"""
        if 'left' in button_str.lower():
            return Button.left
        elif 'right' in button_str.lower():
            return Button.right
        elif 'middle' in button_str.lower():
            return Button.middle
        return Button.left
    
    def _parse_key(self, key_str):
        """Convierte string a Key"""
        # Mapeo de teclas especiales
        special_keys = {
            'Key.space': Key.space,
            'Key.enter': Key.enter,
            'Key.tab': Key.tab,
            'Key.backspace': Key.backspace,
            'Key.esc': Key.esc,
            'Key.shift': Key.shift,
            'Key.shift_r': Key.shift_r,
            'Key.ctrl': Key.ctrl,
            'Key.ctrl_r': Key.ctrl_r,
            'Key.alt': Key.alt,
            'Key.alt_r': Key.alt_r,
            'Key.cmd': Key.cmd,
            'Key.up': Key.up,
            'Key.down': Key.down,
            'Key.left': Key.left,
            'Key.right': Key.right,
        }
        
        if key_str in special_keys:
            return special_keys[key_str]
        
        # Si es un carácter normal
        return key_str
