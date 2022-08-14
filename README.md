# Template Pallet

## Overview

FRAME pallet to manage and store music styles on-chain. Styles are divided into main styles that could hold sub styles.
Each style has a name and an id, name could be changed but the id is immutable.

## Interface

### Dispatchable functions

#### Getters

- `get` - returns all music styles.
- `contains` - search for a style or sub style by id (hash) and returns a boolean.

#### For admin users

- `add` - Store a new music style (and sub styles).
- `add_sub_style` - Store a new music sub style into a primary style.
- `update_style_name` - Update a first level style name.
- `remove` - Remove a music style (add related sub styles) or a sub style.

License: Unlicense