---
author:
  - name: Ada Lovelace
    id: ada
    institute: [cam]
  - name: Charles Babbage
    institute: University of Cambridge
  - Grace Hopper: Yale
  - Alan Turing:
      email: alan@example.org
      institute: [cam, mit, yale]
  - name: Emmy Noether
    id: false
  - Kurt Gödel:
      id: kg
      institute:
        name: Institute for Advanced Study
        address:
          street: 1 Einstein Drive
          city: Princeton
  - Hedy Lamarr:
      name: false
institute:
  - cam:
      name: University of Cambridge
  - mit: Massachusetts Institute of Technology
  - name: Yale University
    id: yale
---

Named objects of every kind: a `name` (an `id` too, or a false one), a
single entry whose value is a name or a map, and institutes referred to
by identifier, by name, or given in place.
