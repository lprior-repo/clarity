with open('clarity-web/src/intent/beads/feedback.rs', 'r') as f:
    content = f.read()

content = content.replace('once_cell::sync::Lazy', 'std::sync::LazyLock')

with open('clarity-web/src/intent/beads/feedback.rs', 'w') as f:
    f.write(content)

with open('clarity-web/src/lattice/coverage.rs', 'r') as f:
    content = f.read()

content = content.replace('use once_cell::sync::Lazy;', 'use std::sync::LazyLock as Lazy;')

with open('clarity-web/src/lattice/coverage.rs', 'w') as f:
    f.write(content)

