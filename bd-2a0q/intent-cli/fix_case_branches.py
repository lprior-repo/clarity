#!/usr/bin/env python3
"""
Fix case expressions in rules_engine_test.gleam.
Add braces to case branches that have multiple statements or nested cases.
"""

import re

def count_statements(lines, start_idx, base_indent):
    """
    Count the number of statements in a case branch.
    Returns (statement_count, has_nested_case, end_idx)
    """
    count = 0
    has_nested = False
    i = start_idx

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Empty or comment - skip
        if not stripped or stripped.startswith('//'):
            i += 1
            continue

        # End of case block
        if stripped == '}' and len(line) - len(line.lstrip()) <= base_indent:
            break

        # Next pattern in same case
        if '->' in line and len(line) - len(line.lstrip()) == base_indent + 2:
            break

        # Nested case
        if stripped.startswith('case '):
            has_nested = True
            count += 1
        # Regular statement
        elif not stripped.startswith('case '):
            count += 1

        i += 1

    return count, has_nested, i

def needs_braces(lines, idx):
    """Check if a case branch needs braces."""
    line = lines[idx]

    # Already has braces
    if '-> {' in line or '->\t{' in line or '->  {' in line:
        return False, []

    # Get the base indent
    base_indent = len(line) - len(line.lstrip())

    # Count statements
    count, has_nested, end_idx = count_statements(lines, idx + 1, base_indent)

    # Needs braces if multiple statements OR has nested case
    # UNLESS it's just one statement that's not a nested case
    if count > 1:
        return True, lines[idx+1:end_idx]

    if has_nested and count == 1:
        # Single nested case - technically could be without braces
        # but for clarity let's check if it's the only thing
        # Actually in Gleam, a single expression doesn't need braces
        # So if it's just a nested case and nothing else, no braces needed
        # But if there are other statements before/after, braces needed
        return False, []

    return False, []

def fix_file(filename):
    """Fix the file."""
    with open(filename, 'r') as f:
        lines = f.readlines()

    result = []
    i = 0

    while i < len(lines):
        line = lines[i]
        result.append(line)

        # Check if this is a case pattern with ->
        if '->' in line and not line.strip().startswith('//'):
            needs, branch_lines = needs_braces(lines, i)

            if needs:
                # Add opening brace
                result[-1] = line.rstrip() + ' {\n'

                # Add branch lines with proper indentation
                for bl in branch_lines:
                    result.append(bl)

                # Add closing brace before next pattern
                result.append('  }\n')

                # Skip the branch lines we just added
                i += len(branch_lines)

        i += 1

    return ''.join(result)

# Fix the file
fixed = fix_file('test/intent/rules_engine_test.gleam')

with open('test/intent/rules_engine_test.gleam', 'w') as f:
    f.write(fixed)

print("Fixed case expressions!")
