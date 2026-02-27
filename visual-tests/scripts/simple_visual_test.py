#!/usr/bin/env python3
"""
Simple Visual Test for Axiom GUI
Uses pyautogui to interact with the GUI and capture screenshots
"""

import os
import sys
import time
import subprocess
from pathlib import Path
from datetime import datetime

# Check if we can use pyautogui
try:
    import pyautogui
    HAS_PYAUTOGUI = True
except ImportError:
    HAS_PYAUTOGUI = False
    print("WARNING: pyautogui not available, will use basic screenshot approach")

class SimpleVisualTest:
    def __init__(self, base_dir):
        self.base_dir = Path(base_dir)
        self.structures_dir = self.base_dir / "structures"
        self.screenshots_dir = self.base_dir / "screenshots"
        self.screenshots_dir.mkdir(exist_ok=True)

        self.gui_process = None
        self.results = []

    def take_screenshot(self, name, description=""):
        """Take screenshot using scrot or imagemagick"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{name}.png"
        filepath = self.screenshots_dir / filename

        # Use scrot with DISPLAY=:99
        try:
            subprocess.run(
                ["scrot", str(filepath)],
                env={**os.environ, "DISPLAY": ":99"},
                check=True,
                timeout=5
            )
            print(f"  Screenshot: {filename}")
            self.results.append({
                "name": name,
                "description": description,
                "file": filename,
                "success": True
            })
            return filepath
        except Exception as e:
            print(f"  Screenshot FAILED: {e}")
            self.results.append({
                "name": name,
                "description": description,
                "error": str(e),
                "success": False
            })
            return None

    def start_gui(self):
        """Start the Axiom GUI in development mode"""
        print("\n=== Starting Axiom GUI ===")

        # Change to GUI directory
        gui_dir = Path("/home/agent/projects/axiom/axiom-gui")

        # Start in development mode
        env = {**os.environ, "DISPLAY": ":99"}

        self.gui_process = subprocess.Popen(
            ["npm", "run", "tauri", "dev"],
            cwd=gui_dir,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )

        print("  Waiting for GUI to start...")
        time.sleep(15)  # Wait for Tauri to build and launch
        print("  GUI should be running")

    def stop_gui(self):
        """Stop the GUI"""
        if self.gui_process:
            print("\n=== Stopping GUI ===")
            self.gui_process.terminate()
            try:
                self.gui_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.gui_process.kill()

    def run_visual_tests(self):
        """Run all visual tests"""
        print("\n=== Running Visual Tests ===")

        # Test 1: Initial state
        print("\n[Test 1] Initial GUI state")
        time.sleep(2)
        self.take_screenshot("01_initial_state", "GUI initial state after launch")

        # Test 2: Wait a bit for any auto-rendering
        print("\n[Test 2] After 5 seconds")
        time.sleep(5)
        self.take_screenshot("02_after_5sec", "GUI state after 5 seconds")

        # Test 3: Try loading a file programmatically if possible
        # For now, just capture current state
        print("\n[Test 3] Current state")
        time.sleep(2)
        self.take_screenshot("03_current_state", "Current GUI state")

        # Test 4-10: Capture more states over time
        for i in range(4, 11):
            print(f"\n[Test {i}] Timed capture")
            time.sleep(3)
            self.take_screenshot(f"{i:02d}_timed_capture", f"GUI state at test {i}")

    def generate_report(self):
        """Generate simple report"""
        report_path = self.base_dir / "SIMPLE_VALIDATION_REPORT.md"

        with open(report_path, 'w') as f:
            f.write("# Axiom GUI Simple Visual Validation\n\n")
            f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")

            f.write("## Summary\n\n")
            successful = sum(1 for r in self.results if r.get('success', False))
            f.write(f"- Screenshots Captured: {successful}/{len(self.results)}\n\n")

            f.write("## Test Structures Available\n\n")
            for struct_file in sorted(self.structures_dir.glob("*")):
                f.write(f"- `{struct_file.name}`\n")

            f.write("\n## Screenshots\n\n")
            for result in self.results:
                if result.get('success', False):
                    f.write(f"### {result['name']}\n")
                    if result['description']:
                        f.write(f"{result['description']}\n\n")
                    f.write(f"![{result['name']}](screenshots/{result['file']})\n\n")
                else:
                    f.write(f"### {result['name']} (FAILED)\n")
                    f.write(f"Error: {result.get('error', 'Unknown')}\n\n")

        print(f"\nReport generated: {report_path}")
        return report_path

def main():
    base_dir = Path("/home/agent/projects/axiom/axiom-gui/visual-tests")

    tester = SimpleVisualTest(base_dir)

    try:
        tester.start_gui()
        tester.run_visual_tests()
        tester.generate_report()
    finally:
        tester.stop_gui()

    print("\n=== Testing Complete ===")
    print(f"Screenshots captured: {sum(1 for r in tester.results if r.get('success', False))}")
    print(f"Check: {base_dir}/screenshots/")

if __name__ == "__main__":
    main()
