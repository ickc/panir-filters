---
author:
  - name: Ann Poe
    equal_contributor: no
    email: ann@example.org
  - name: Bo *Lee*
    correspondence: false
    email: bo@example.org
    institute: [a, b]
  - name: Cy Ray
    correspondence: yes
    institute: b
  - name: Di Fox
    correspondence: 1
    email: "[di](mailto:di@example.org)"
    equal_contributor: true
institute:
  - a: First Institute
  - b:
      name: Second Institute
      address: Elsewhere
---

Marks: "no" is true (it is text), false is false, correspondence needs an
email, an email may hold a link.
