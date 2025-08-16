# MOBA — EZ Mode sample (Kids‑friendly)

Place Player at [0,1,0] has Hero
Place Camera at [0,6,12]
Set speed = 6
Set hp = 100

When key W:
  Move Player forward by speed
When key S:
  Move Player back by speed
When key A:
  Move Player left by speed
When key D:
  Move Player right by speed

When press Space:
  Let dash = speed*3
  Move Player forward by dash
  Set dash_cd = 1s

Every 0.1s:
  Show "HP: {hp}" on HpText 