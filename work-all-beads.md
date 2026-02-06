Work through ALL beads in the repository systematically. The beads are stored in .beads/beads.db. Process them in priority order (lowest first), starting with foundation items then core then web features.

For each bead:
1. Read the bead details from the database
2. Implement the requirements
3. Test the implementation
4. Mark the bead as complete in the database
5. Move to the next bead
6. Continue until ALL 42 beads are completed

The beads are organized as:
- Foundation beads (bd-2ck, bd-2b3, bd-3s0, bd-1ib, bd-26p, bd-3vg, bd-2if, bd-1ft, bd-di8)
- Core beads (bd-1my, bd-ezl, bd-3ue, bd-3ki, bd-2lb, bd-4h9, bd-2fg, bd-3ey, bd-264, bd-2yt, bd-dws, bd-1v2, bd-57v, bd-2nx)
- Web beads (bd-21h, bd-1bc, bd-z11, bd-2cg, bd-4pq, bd-3rr, bd-84g, bd-w5a, bd-1r8, bd-3n6, bd-3j3, bd-ccj, bd-3tq, bd-34e, bd-2j4, bd-4a6, bd-2pj, bd-10g)

Use SQLite to query and update bead status. Update the status field from 'open' to 'closed' when complete. Use completion promise "ALL BEADS COMPLETED" when done.

Work through each bead completely - don't skip any. Focus on high-quality, tested code that follows the zero-unwrap philosophy documented in AGENTS.md.