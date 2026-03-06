with open('clarity-web/src/intent/validation/semantic_bdd_tests.rs', 'r') as f:
    content = f.read()

import re

content = re.sub(
    r'(auth|users|orders)\.add_behavior\((.*?)\.unwrap\(\)\);',
    r'let _ = \1.add_behavior(\2.unwrap());',
    content
)

with open('clarity-web/src/intent/validation/semantic_bdd_tests.rs', 'w') as f:
    f.write(content)

with open('clarity-web/tests/queue_adversarial.rs', 'r') as f:
    queue = f.read()

if '#![allow(suspicious_double_ref_op)]' not in queue:
    queue = '#![allow(suspicious_double_ref_op)]\n' + queue

with open('clarity-web/tests/queue_adversarial.rs', 'w') as f:
    f.write(queue)

with open('clarity-web/tests/adversarial_gen2_quality.rs', 'r') as f:
    quality = f.read()

if '#![allow(unused_comparisons)]' not in quality:
    quality = '#![allow(unused_comparisons)]\n' + quality

with open('clarity-web/tests/adversarial_gen2_quality.rs', 'w') as f:
    f.write(quality)
