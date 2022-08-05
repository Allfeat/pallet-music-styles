# Template Pallet

## Overview

FRAME pallet to manage and store music styles on-chain.

## Interface

### Dispatchable functions

#### Getters

- `get` - returns all music styles.

#### For admin users

- `add` - Store a new music style (and sub styles).
- `add_sub_style` - Store a new music sub style into a primary style.
- `update_style_name` - Update a first level style name.
- `remove` - Remove a music style (add related sub styles).
- `remove_sub_style` - Remove a music sub style from a primary style.

License: Unlicense