<!-- BEGIN COMMENT -->

Hidden at the start.

<!-- END COMMENT -->

Kept, with an inline <!-- BEGIN COMMENT --> marker that does nothing.

<!-- BEGIN COMMENT -->

Hidden, then a marker that is not html:

```{=latex}
<!-- END COMMENT -->
```

Still hidden.

<!-- END COMMENT -->

<!-- END COMMENT -->

Kept: an end without a beginning is dropped, nothing else.

```{=html}
<section>
<!-- BEGIN COMMENT -->
</section>
```

Hidden: a beginning anywhere in a raw block counts.

<!-- END COMMENT -->

Kept.

```{=html}
<!-- BEGIN COMMENT --><!-- END COMMENT -->
```

Hidden: one block with both is a beginning.

<!-- END COMMENT -->

Kept.

<div class="box">
<!-- BEGIN COMMENT -->
</div>

Hidden: the div stays, empty.

<!-- END COMMENT -->

Kept at the end.

<!-- BEGIN COMMENT -->
