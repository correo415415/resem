# Reference screenshots (original Flash game)

User-provided captures of the original Resort Empire (Little Giant World),
used as ground truth for the port.

- `original_toolbar_tutorial_locked.png` — during the tutorial: every toolbar
  tool is locked (baked lock frames) except the arrow and drag tools. The
  drag tool still shows a small lock badge at this point. Tutorial blackboard
  dialog + "Click here to skip tutorial" button visible. HUD: RP (0%),
  counters at 0, DAY 1, $10000, POPULARITY 0%.
- `original_toolbar_post_tutorial.png` — right after the tutorial: all tools
  unlocked (icons visible) except Expand, which keeps its small lock badge.
  Tips shows the NEW badge. Mission ticker on nav3 reads
  "Build a room on your resort" with a gift reward of $500.

Notes for the port:
- Our current start state (skip tutorial) must match the post-tutorial
  capture: all icons unlocked, Expand locked, NEW on Tips.
- During-tutorial progressive unlocking is a future feature tied to the
  tutorial system (tips dialog + mission flow).
