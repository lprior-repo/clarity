import re

with open('clarity-web/src/intent/templates/spec_templates.rs', 'r') as f:
    content = f.read()

# Make sure Answer is actually imported if it got reverted
if 'use crate::intent::interview::types::{InterviewSession, Profile};' in content:
    content = content.replace('use crate::intent::interview::types::{InterviewSession, Profile};', 'use crate::intent::interview::types::{Answer, InterviewSession, Profile};')
elif 'use crate::intent::interview::types::{Answer, InterviewSession, Profile};' not in content:
    content = re.sub(
        r'(use crate::intent::interview::types::\{.*?)(?=\};)',
        r'\1, Answer',
        content
    )

with open('clarity-web/src/intent/templates/spec_templates.rs', 'w') as f:
    f.write(content)

with open('clarity-web/src/intent/plan/plan_next.rs', 'r') as f:
    plan_next = f.read()

plan_next = plan_next.replace('use crate::intent::interview::types::{Answer, Conflict, ConflictResolution, Gap, Profile};', 'use crate::intent::interview::types::{Conflict, ConflictResolution, Gap, Profile};')
plan_next = plan_next.replace('use std::collections::HashMap;\n', '')

with open('clarity-web/src/intent/plan/plan_next.rs', 'w') as f:
    f.write(plan_next)
    
with open('clarity-web/src/intent/parser.rs', 'r') as f:
    parser = f.read()

parser = parser.replace('expected,', 'expected: _,')

with open('clarity-web/src/intent/parser.rs', 'w') as f:
    f.write(parser)
