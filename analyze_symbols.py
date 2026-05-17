#!/usr/bin/env python3
import re

swiftinterface = """$(cat "$SDK/System/Library/Frameworks/StoreKit.framework/Versions/A/Modules/StoreKit.swiftmodule/arm64e-apple-macos.swiftinterface")"""

# Count all "public" declarations
all_public = len(re.findall(r'\bpublic\s+', swiftinterface))
print(f"Total 'public' declarations: {all_public}")

# Count specific kinds
for kind in ['struct', 'enum', 'class', 'protocol', 'func', 'var', 'typealias']:
    count = len(re.findall(rf'\bpublic\s+{kind}\b', swiftinterface))
    print(f"{kind}: {count}")

# Count unavailable
unavailable = len(re.findall(r'@available\([^)]*unavailable', swiftinterface))
deprecated = len(re.findall(r'@available\([^)]*deprecated', swiftinterface))
print(f"\nUnavailable: {unavailable}")
print(f"Deprecated: {deprecated}")
