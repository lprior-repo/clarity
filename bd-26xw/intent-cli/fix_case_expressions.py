#!/usr/bin/env python3
"""Fix case expressions in rules_engine_test.gleam to add braces where needed."""

import re
import sys

def fix_case_expressions(content):
    """Add braces to case expressions with multiple statements."""

    # Split into lines
    lines = content.split('\n')
    i = 0
    result = []

    while i < len(lines):
        line = lines[i]
        result.append(line)

        # Check if this line has a case expression pattern with ->
        if '->' in line and not line.strip().startswith('//'):
            # Check if there are braces after ->
            if '-> {' in line or '->\t{' in line or '->  {' in line:
                # Already has braces, skip
                i += 1
                continue

            # Count statements until next pattern (lines starting with a word and then ->)
            # or closing brace
            statement_count = 0
            j = i + 1
            has_nested_case = False

            while j < len(lines):
                next_line = lines[j].strip()

                # Empty lines or comments don't count
                if not next_line or next_line.startswith('//'):
                    j += 1
                    continue

                # Closing brace ends the block
                if next_line == '}':
                    break

                # Next pattern starts
                if re.match(r'^[\w\[\]_,\s]+->', next_line):
                    break

                # Check for nested case
                if next_line.startswith('case '):
                    has_nested_case = True
                    statement_count += 1
                    j += 1
                    continue

                # Regular statement
                statement_count += 1
                j += 1

            # If we have multiple statements or a nested case, we need braces
            if statement_count > 1:
                # Add opening brace after ->
                result[-1] = line + ' {'

                # Find the line before the next pattern and add closing brace
                insert_pos = len(result)
                k = i + 1
                while k < len(lines):
                    next_line = lines[k].strip()
                    if not next_line or next_line.startswith('//'):
                        result.append(lines[k])
                        k += 1
                        continue

                    if next_line == '}':
                        # Insert closing brace before this
                        result.insert(insert_pos, '  }')
                        break

                    if re.match(r'^[\w\[\]_,\s]+->', next_line):
                        # Insert closing brace before this pattern
                        result.insert(insert_pos, '  }')
                        break

                    result.append(lines[k])
                    k += 1

                # Skip lines we already added
                i = k - 1

        i += 1

    return '\n'.join(result)

# Read the file
with open('test/intent/rules_engine_test.gleam', 'r') as f:
    content = f.read()

# Fix it
fixed_content = fix_case_expressions(content)

# Write it back
with open('test/intent/rules_engine_test.gleam', 'w') as f:
    f.write(fixed_content)

print("Fixed case expressions in test/intent/rules_engine_test.gleam")
