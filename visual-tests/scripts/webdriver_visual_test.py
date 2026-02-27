#!/usr/bin/env python3
"""
WebDriver-based Visual Test for Axiom GUI
Uses WebDriver to interact with the Tauri app's WebView
"""

import os
import sys
import time
import json
import subprocess
from pathlib import Path
from datetime import datetime
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.common.action_chains import ActionChains

class WebDriverVisualTest:
    def __init__(self, base_dir):
        self.base_dir = Path(base_dir)
        self.structures_dir = self.base_dir / "structures"
        self.screenshots_dir = self.base_dir / "screenshots"
        self.screenshots_dir.mkdir(exist_ok=True)

        self.driver = None
        self.app_url = "http://localhost:1420"  # Tauri dev server default
        self.results = {
            "tests": [],
            "screenshots": [],
            "bugs": []
        }

    def setup_driver(self):
        """Setup Chrome WebDriver for Xvfb"""
        print("Setting up WebDriver...")

        options = webdriver.ChromeOptions()
        options.add_argument('--no-sandbox')
        options.add_argument('--disable-dev-shm-usage')
        options.add_argument('--disable-gpu')
        options.add_argument('--window-size=1920,1080')

        # Set DISPLAY environment variable
        os.environ['DISPLAY'] = ':99'

        try:
            self.driver = webdriver.Chrome(options=options)
            self.driver.set_window_size(1920, 1080)
            print("  WebDriver ready")
            return True
        except Exception as e:
            print(f"  WebDriver setup failed: {e}")
            return False

    def close_driver(self):
        """Close the WebDriver"""
        if self.driver:
            self.driver.quit()

    def take_screenshot(self, name, description=""):
        """Take a screenshot"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{name}.png"
        filepath = self.screenshots_dir / filename

        try:
            self.driver.save_screenshot(str(filepath))
            print(f"  Screenshot: {filename}")

            self.results["screenshots"].append({
                "name": name,
                "description": description,
                "file": filename,
                "timestamp": timestamp
            })

            return filepath
        except Exception as e:
            print(f"  Screenshot failed: {e}")
            return None

    def wait_for_element(self, by, value, timeout=10):
        """Wait for element to be present"""
        try:
            element = WebDriverWait(self.driver, timeout).until(
                EC.presence_of_element_located((by, value))
            )
            return element
        except:
            return None

    def test_gui_loads(self):
        """Test that GUI loads successfully"""
        test_name = "GUI Loading"
        print(f"\n=== Test: {test_name} ===")

        try:
            self.driver.get(self.app_url)
            time.sleep(3)

            self.take_screenshot("00_gui_loaded", "GUI initial load")

            # Check if canvas exists
            canvas = self.driver.find_elements(By.TAG_NAME, "canvas")

            if canvas:
                print(f"  ✓ Canvas found: {len(canvas)} canvas element(s)")
                result = "PASS"
            else:
                print("  ✗ No canvas found")
                result = "FAIL"
                self.results["bugs"].append({
                    "test": test_name,
                    "issue": "No canvas element found in DOM"
                })

            self.results["tests"].append({
                "name": test_name,
                "result": result
            })

            return result == "PASS"

        except Exception as e:
            print(f"  ✗ FAILED: {e}")
            self.results["tests"].append({
                "name": test_name,
                "result": "ERROR",
                "error": str(e)
            })
            self.results["bugs"].append({
                "test": test_name,
                "issue": str(e)
            })
            return False

    def test_load_structure(self, structure_file):
        """Test loading a structure file"""
        test_name = f"Load Structure: {structure_file.name}"
        print(f"\n=== Test: {test_name} ===")

        try:
            # Look for file input element
            file_inputs = self.driver.find_elements(By.CSS_SELECTOR, "input[type='file']")

            if not file_inputs:
                print("  ⚠ No file input found - checking for load button")
                # Try to find a load/open button
                buttons = self.driver.find_elements(By.TAG_NAME, "button")
                for btn in buttons:
                    text = btn.text.lower()
                    if 'load' in text or 'open' in text or 'file' in text:
                        print(f"  Found button: {btn.text}")
                        btn.click()
                        time.sleep(1)
                        # Try again for file input
                        file_inputs = self.driver.find_elements(By.CSS_SELECTOR, "input[type='file']")
                        break

            if file_inputs:
                print(f"  Found {len(file_inputs)} file input(s)")
                file_input = file_inputs[0]

                # Send the file path
                file_input.send_keys(str(structure_file.absolute()))
                print(f"  Sent file: {structure_file.name}")

                # Wait for rendering
                time.sleep(5)

                self.take_screenshot(
                    f"structure_{structure_file.stem}",
                    f"Loaded structure: {structure_file.name}"
                )

                result = "PASS"
            else:
                print("  ✗ Could not find file input")
                result = "FAIL"
                self.results["bugs"].append({
                    "test": test_name,
                    "issue": "No file input element found"
                })

            self.results["tests"].append({
                "name": test_name,
                "result": result
            })

            return result == "PASS"

        except Exception as e:
            print(f"  ✗ FAILED: {e}")
            self.results["tests"].append({
                "name": test_name,
                "result": "ERROR",
                "error": str(e)
            })
            return False

    def test_rotation(self):
        """Test rotation by dragging"""
        test_name = "Rotation"
        print(f"\n=== Test: {test_name} ===")

        try:
            canvas = self.driver.find_element(By.TAG_NAME, "canvas")

            # Before rotation
            self.take_screenshot("rotation_before", "Before rotation")

            # Perform drag
            actions = ActionChains(self.driver)
            actions.move_to_element(canvas)
            actions.click_and_hold()
            actions.move_by_offset(200, 100)
            actions.release()
            actions.perform()

            time.sleep(2)

            # After rotation
            self.take_screenshot("rotation_after", "After rotation (drag 200x100)")

            print("  ✓ Rotation test completed (visual comparison needed)")

            self.results["tests"].append({
                "name": test_name,
                "result": "VISUAL",
                "note": "Compare before/after screenshots"
            })

            return True

        except Exception as e:
            print(f"  ✗ FAILED: {e}")
            self.results["tests"].append({
                "name": test_name,
                "result": "ERROR",
                "error": str(e)
            })
            return False

    def test_controls_visible(self):
        """Test that GUI controls are visible"""
        test_name = "UI Controls Visibility"
        print(f"\n=== Test: {test_name} ===")

        try:
            # Capture full page
            self.take_screenshot("ui_controls", "UI controls overview")

            # Look for common control elements
            controls_found = []

            # Check for buttons
            buttons = self.driver.find_elements(By.TAG_NAME, "button")
            if buttons:
                controls_found.append(f"{len(buttons)} buttons")

            # Check for inputs
            inputs = self.driver.find_elements(By.TAG_NAME, "input")
            if inputs:
                controls_found.append(f"{len(inputs)} inputs")

            # Check for checkboxes
            checkboxes = self.driver.find_elements(By.CSS_SELECTOR, "input[type='checkbox']")
            if checkboxes:
                controls_found.append(f"{len(checkboxes)} checkboxes")

            # Check for select elements
            selects = self.driver.find_elements(By.TAG_NAME, "select")
            if selects:
                controls_found.append(f"{len(selects)} select boxes")

            print(f"  Controls found: {', '.join(controls_found)}")

            # Get page source for detailed analysis
            page_source = self.driver.page_source

            # Check for specific keywords in UI
            keywords = ['rotation', 'zoom', 'background', 'render', 'load', 'open', 'file']
            found_keywords = [kw for kw in keywords if kw.lower() in page_source.lower()]

            if found_keywords:
                print(f"  UI keywords found: {', '.join(found_keywords)}")

            self.results["tests"].append({
                "name": test_name,
                "result": "PASS",
                "controls": controls_found,
                "keywords": found_keywords
            })

            return True

        except Exception as e:
            print(f"  ✗ FAILED: {e}")
            self.results["tests"].append({
                "name": test_name,
                "result": "ERROR",
                "error": str(e)
            })
            return False

    def generate_report(self):
        """Generate visual validation report"""
        report_path = self.base_dir / "VISUAL_VALIDATION_REPORT.md"

        with open(report_path, 'w') as f:
            f.write("# Axiom GUI Visual Validation Report\n\n")
            f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")

            # Summary
            f.write("## Summary\n\n")
            total_tests = len(self.results["tests"])
            passed = sum(1 for t in self.results["tests"] if t["result"] == "PASS")
            failed = sum(1 for t in self.results["tests"] if t["result"] == "FAIL")
            errors = sum(1 for t in self.results["tests"] if t["result"] == "ERROR")
            visual = sum(1 for t in self.results["tests"] if t["result"] == "VISUAL")

            f.write(f"- Total Tests: {total_tests}\n")
            f.write(f"- Passed: {passed}\n")
            f.write(f"- Failed: {failed}\n")
            f.write(f"- Errors: {errors}\n")
            f.write(f"- Visual Validation Required: {visual}\n")
            f.write(f"- Screenshots Captured: {len(self.results['screenshots'])}\n\n")

            # Test structures
            f.write("## Test Structures\n\n")
            f.write("Small structures (3-20 atoms) for fast rendering:\n\n")
            for struct_file in sorted(self.structures_dir.glob("*")):
                # Count atoms
                if struct_file.suffix == '.xyz':
                    with open(struct_file) as sf:
                        atom_count = sf.readline().strip()
                    f.write(f"- **{struct_file.name}**: {atom_count} atoms\n")
                else:
                    f.write(f"- **{struct_file.name}**\n")
            f.write("\n")

            # Test results
            f.write("## Test Results\n\n")
            for test in self.results["tests"]:
                f.write(f"### {test['name']}\n")
                f.write(f"**Result**: {test['result']}\n\n")
                if "note" in test:
                    f.write(f"*{test['note']}*\n\n")
                if "error" in test:
                    f.write(f"```\n{test['error']}\n```\n\n")
                if "controls" in test:
                    f.write(f"Controls: {', '.join(test['controls'])}\n\n")
                if "keywords" in test:
                    f.write(f"UI Keywords: {', '.join(test['keywords'])}\n\n")

            # Bugs
            if self.results["bugs"]:
                f.write("## Bugs / Issues Found\n\n")
                for i, bug in enumerate(self.results["bugs"], 1):
                    f.write(f"### {i}. {bug['test']}\n")
                    f.write(f"{bug['issue']}\n\n")

            # Screenshots
            f.write("## Screenshots\n\n")
            for ss in self.results["screenshots"]:
                f.write(f"### {ss['name']}\n")
                if ss['description']:
                    f.write(f"{ss['description']}\n\n")
                f.write(f"![{ss['name']}](screenshots/{ss['file']})\n\n")
                f.write(f"*Timestamp: {ss['timestamp']}*\n\n")

        print(f"\n✓ Report generated: {report_path}")
        return report_path

def main():
    base_dir = Path("/home/agent/projects/axiom/axiom-gui/visual-tests")

    print("=== Axiom GUI Visual Validation ===\n")
    print("Prerequisites:")
    print("  - Xvfb running on :99")
    print("  - Axiom GUI dev server on http://localhost:1420")
    print("  - ChromeDriver installed\n")

    tester = WebDriverVisualTest(base_dir)

    try:
        if not tester.setup_driver():
            print("\nERROR: Could not setup WebDriver")
            return 1

        # Run tests
        tester.test_gui_loads()
        tester.test_controls_visible()

        # Try loading structures
        for struct in ["water.xyz", "methane.xyz"]:
            struct_file = tester.structures_dir / struct
            if struct_file.exists():
                tester.test_load_structure(struct_file)

        tester.test_rotation()

        # Generate report
        tester.generate_report()

    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
    except Exception as e:
        print(f"\n\nERROR: {e}")
    finally:
        tester.close_driver()

    print("\n=== Validation Complete ===")
    print(f"Screenshots: {len(tester.results['screenshots'])}")
    print(f"Check: {base_dir}/screenshots/")

    return 0

if __name__ == "__main__":
    sys.exit(main())
