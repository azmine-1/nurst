#!/bin/bash
# Compare the first N lines of logs to find where they diverge

if [ ! -f "my_nestest.log" ]; then
    echo "Error: my_nestest.log not found. Run the emulator first."
    exit 1
fi

if [ ! -f "nestest.log" ]; then
    echo "Error: nestest.log not found."
    exit 1
fi

echo "Comparing logs..."
echo ""

# Find first difference
diff -y --suppress-common-lines my_nestest.log nestest.log | head -20

echo ""
echo "Summary:"
DIFF_LINE=$(diff my_nestest.log nestest.log | head -1 | grep -oP '^\d+')
if [ -z "$DIFF_LINE" ]; then
    echo "✓ Logs match perfectly!"
else
    echo "✗ First difference at line $DIFF_LINE"
    echo ""
    echo "Expected (nestest.log):"
    sed -n "${DIFF_LINE}p" nestest.log
    echo ""
    echo "Got (my_nestest.log):"
    sed -n "${DIFF_LINE}p" my_nestest.log
fi
