---
author:
  - A:
      institute:
        - name: New Lab
          city: Here
          open: false
  - B:
      institute:
        - name: New Lab
          country: There
          open: true
        - fosg
  - C:
      institute: [fosg, fosg]
  - D:
      institute: 2, x
institute:
  - fosg: Formatting Open Science Group
  - fosg:
      name: Another FOSG
      address: Somewhere
---

Institutes given in two authors merge (a false entry counts as missing);
two top-level ones with one identifier stay two, and references go to the
first, which takes what the second has that it lacks when the second is
referred to by number.
