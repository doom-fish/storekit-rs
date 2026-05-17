#!/usr/bin/env python3
"""
Verify StoreKit 2 audit by reading swiftinterface and comparing against COVERAGE_AUDIT.md
"""
import re

# Read the swiftinterface file
with open(f"{SDK}/System/Library/Frameworks/StoreKit.framework/Versions/A/Modules/StoreKit.swiftmodule/arm64e-apple-macos.swiftinterface", "r") as f:
    content = f.read()

# Count public declarations
# Top-level items
top_level = re.findall(r'^public\s+(struct|enum|class|protocol|typealias)\s+(\w+)', content, re.MULTILINE)
print(f"Top-level public items: {len(top_level)}")
for kind, name in top_level[:10]:
    print(f"  {kind} {name}")

# Count @available unavailable (macOS unavailable)
unavailable_lines = re.findall(r'@available\([^)]*unavailable\)', content)
print(f"\nUnavailable items: {len(unavailable_lines)}")

# Count @available deprecated
deprecated_matches = re.findall(r'@available\([^)]*deprecated[^)]*\)', content)
print(f"Deprecated items: {len(deprecated_matches)}")

# Check for key symbols from v1 audit
key_symbols = [
    'public struct Product',
    'public struct Transaction',
    'public struct Storefront',
    'public struct AppTransaction',
    'public enum AppStore',
    'public enum ExternalPurchase',
    'public struct Message',
    'public protocol StoreDownloaderExtension',
]

print("\n=== Key symbols from v1 audit ===")
for sym in key_symbols:
    if sym in content:
        print(f"✓ {sym}")
    else:
        print(f"✗ {sym} - NOT FOUND")
