#!/bin/bash
# Extract and count public symbols from SwiftInterfaces
SWIFTINTERFACE="$1"

# Count top-level public items
echo "=== TOP-LEVEL PUBLIC SYMBOLS ==="
grep -E "^public (class|struct|enum|protocol|func|var|typealias) " "$SWIFTINTERFACE" | wc -l

# Count per type
echo ""
echo "=== BY KIND ==="
echo "Classes: $(grep -c "^public class " "$SWIFTINTERFACE")"
echo "Structs: $(grep -c "^public struct " "$SWIFTINTERFACE")"
echo "Enums: $(grep -c "^public enum " "$SWIFTINTERFACE")"
echo "Protocols: $(grep -c "^public protocol " "$SWIFTINTERFACE")"
echo "Functions: $(grep -c "^public func " "$SWIFTINTERFACE")"
echo "Vars: $(grep -c "^public var " "$SWIFTINTERFACE")"
echo "Typealiases: $(grep -c "^public typealias " "$SWIFTINTERFACE")"

# Count unavailable/deprecated
echo ""
echo "=== AVAILABILITY ==="
grep -B1 "@available.*unavailable" "$SWIFTINTERFACE" | grep -E "^public " | wc -l | xargs echo "Unavailable items:"
grep -B1 "@available.*deprecated" "$SWIFTINTERFACE" | grep -E "^public " | wc -l | xargs echo "Deprecated items:"

