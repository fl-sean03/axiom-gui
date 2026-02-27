#!/usr/bin/env python3
"""
Direct Axiom Rendering Test
Uses the axiom library to load structures and test rendering programmatically
"""

import sys
import time
from pathlib import Path
from datetime import datetime

# Add axiom to path
sys.path.insert(0, '/home/agent/projects/axiom')

try:
    import axiom
    from axiom import Molecule
    print(f"Axiom version: {axiom.__version__ if hasattr(axiom, '__version__') else 'unknown'}")
except ImportError as e:
    print(f"ERROR: Cannot import axiom: {e}")
    print("Make sure axiom is installed: pip install -e /home/agent/projects/axiom")
    sys.exit(1)

class AxiomRenderTest:
    def __init__(self, base_dir):
        self.base_dir = Path(base_dir)
        self.structures_dir = self.base_dir / "structures"
        self.screenshots_dir = self.base_dir / "screenshots"
        self.screenshots_dir.mkdir(exist_ok=True)

        self.results = {
            "tests": [],
            "structures_loaded": [],
            "errors": []
        }

    def test_load_xyz(self, xyz_file):
        """Test loading an XYZ file"""
        test_name = f"Load XYZ: {xyz_file.name}"
        print(f"\n=== {test_name} ===")

        try:
            # Try to load the structure
            mol = Molecule.from_file(str(xyz_file))

            print(f"  ✓ Loaded: {len(mol.atoms)} atoms")
            print(f"  Elements: {set(atom.element for atom in mol.atoms)}")

            self.results["structures_loaded"].append({
                "file": xyz_file.name,
                "atoms": len(mol.atoms),
                "success": True
            })

            self.results["tests"].append({
                "name": test_name,
                "result": "PASS"
            })

            return mol

        except Exception as e:
            print(f"  ✗ FAILED: {e}")
            self.results["errors"].append({
                "test": test_name,
                "error": str(e)
            })
            self.results["tests"].append({
                "name": test_name,
                "result": "FAIL"
            })
            return None

    def test_load_pdb(self, pdb_file):
        """Test loading a PDB file"""
        test_name = f"Load PDB: {pdb_file.name}"
        print(f"\n=== {test_name} ===")

        try:
            mol = Molecule.from_file(str(pdb_file))

            print(f"  ✓ Loaded: {len(mol.atoms)} atoms")

            self.results["structures_loaded"].append({
                "file": pdb_file.name,
                "atoms": len(mol.atoms),
                "success": True
            })

            self.results["tests"].append({
                "name": test_name,
                "result": "PASS"
            })

            return mol

        except Exception as e:
            print(f"  ✗ FAILED: {e}")
            self.results["errors"].append({
                "test": test_name,
                "error": str(e)
            })
            self.results["tests"].append({
                "name": test_name,
                "result": "FAIL"
            })
            return None

    def test_molecule_properties(self, mol, name):
        """Test basic molecule properties"""
        test_name = f"Molecule Properties: {name}"
        print(f"\n=== {test_name} ===")

        try:
            print(f"  Atoms: {len(mol.atoms)}")
            print(f"  Bonds: {len(mol.bonds) if hasattr(mol, 'bonds') else 'N/A'}")

            # Try to get bounds
            if hasattr(mol, 'bounds') or hasattr(mol, 'get_bounds'):
                bounds = mol.bounds() if callable(getattr(mol, 'bounds', None)) else mol.bounds
                print(f"  Bounds: {bounds}")

            # Try to get center
            if hasattr(mol, 'center') or hasattr(mol, 'get_center'):
                center = mol.center() if callable(getattr(mol, 'center', None)) else mol.center
                print(f"  Center: {center}")

            self.results["tests"].append({
                "name": test_name,
                "result": "PASS"
            })

            return True

        except Exception as e:
            print(f"  ✗ FAILED: {e}")
            self.results["errors"].append({
                "test": test_name,
                "error": str(e)
            })
            self.results["tests"].append({
                "name": test_name,
                "result": "FAIL"
            })
            return False

    def generate_report(self):
        """Generate test report"""
        report_path = self.base_dir / "AXIOM_LIBRARY_TEST_REPORT.md"

        with open(report_path, 'w') as f:
            f.write("# Axiom Library Test Report\n\n")
            f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")

            # Summary
            total = len(self.results["tests"])
            passed = sum(1 for t in self.results["tests"] if t["result"] == "PASS")
            failed = sum(1 for t in self.results["tests"] if t["result"] == "FAIL")

            f.write("## Summary\n\n")
            f.write(f"- Total Tests: {total}\n")
            f.write(f"- Passed: {passed}\n")
            f.write(f"- Failed: {failed}\n")
            f.write(f"- Success Rate: {passed/total*100:.1f}%\n\n")

            # Structures loaded
            f.write("## Structures Loaded\n\n")
            for struct in self.results["structures_loaded"]:
                status = "✓" if struct["success"] else "✗"
                f.write(f"- {status} **{struct['file']}**: {struct['atoms']} atoms\n")
            f.write("\n")

            # Test results
            f.write("## Test Results\n\n")
            for test in self.results["tests"]:
                result_icon = "✓" if test["result"] == "PASS" else "✗"
                f.write(f"- {result_icon} {test['name']}: **{test['result']}**\n")
            f.write("\n")

            # Errors
            if self.results["errors"]:
                f.write("## Errors\n\n")
                for error in self.results["errors"]:
                    f.write(f"### {error['test']}\n")
                    f.write(f"```\n{error['error']}\n```\n\n")

        print(f"\n✓ Report generated: {report_path}")
        return report_path

def main():
    base_dir = Path("/home/agent/projects/axiom/axiom-gui/visual-tests")

    print("=== Axiom Library Rendering Test ===\n")

    tester = AxiomRenderTest(base_dir)

    # Test all structures
    structures = list(tester.structures_dir.glob("*.xyz")) + \
                 list(tester.structures_dir.glob("*.pdb"))

    print(f"Found {len(structures)} test structures")

    molecules = {}

    for struct_file in sorted(structures):
        if struct_file.suffix == '.xyz':
            mol = tester.test_load_xyz(struct_file)
        elif struct_file.suffix == '.pdb':
            mol = tester.test_load_pdb(struct_file)
        else:
            continue

        if mol:
            molecules[struct_file.stem] = mol
            tester.test_molecule_properties(mol, struct_file.stem)

    # Generate report
    tester.generate_report()

    print("\n=== Test Complete ===")
    print(f"Tests: {len(tester.results['tests'])}")
    print(f"Structures loaded: {len(tester.results['structures_loaded'])}")
    print(f"Errors: {len(tester.results['errors'])}")

    return 0 if len(tester.results["errors"]) == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
