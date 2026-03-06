with open('clarity-web/src/intent/parser.rs', 'r') as f:
    content = f.read()

content = content.replace('if field == "name" && expected == "string"', 'if field == "name"')
content = content.replace('if field == "root" && expected == "object"', 'if field == "root"')

with open('clarity-web/src/intent/parser.rs', 'w') as f:
    f.write(content)

with open('clarity-web/src/intent/templates/spec_templates.rs', 'r') as f:
    content = f.read()

content = content.replace('use crate::intent::interview::types::{Answer, InterviewSession, Profile};', 'use crate::intent::interview::types::{InterviewSession, Profile};')
if 'mod tests {' in content:
    content = content.replace('use crate::intent::interview::types::{InterviewStage, Perspective};', 'use crate::intent::interview::types::{Answer, InterviewStage, Perspective};')

with open('clarity-web/src/intent/templates/spec_templates.rs', 'w') as f:
    f.write(content)
