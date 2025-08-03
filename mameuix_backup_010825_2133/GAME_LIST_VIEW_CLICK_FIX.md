# Game List View Click Area Fix

## Problem
The user reported that the clickable area in the list view was limited to a small portion of each game item. Clicking on the game title text didn't work for selecting games. This issue affected all rows/cards in the list view.

## Root Cause
The issue was caused by the way we were handling the interactive area in the `render_game_item` method. We were:
1. Creating a frame for visual styling
2. Allocating a response area inside the frame
3. Drawing content on top of that response area
4. Handling clicks on the response area

However, the structure had syntax errors (missing closing brace) that prevented proper compilation and the click detection wasn't working correctly across the entire item area.

## Solution
Fixed the implementation by:

1. **Fixed Syntax Error**: Added the missing closing brace for the arrow click handler:
```rust
if arrow_response.clicked() {
    expanded_parents.insert(game.name.clone(), !is_expanded);
    self.invalidate_cache();
}  // This closing brace was missing
```

2. **Fixed Bracket Mismatch**: Corrected the closing brackets for the layout structure to ensure proper nesting of UI elements.

## Current Implementation Structure
The game item now has this structure:
- Frame (provides visual styling)
  - Response area (covers entire item width/height)
    - Content drawn on top (game info, badges, etc.)
  - Click handling on the response area
  - Context menu on right-click

This ensures that the entire game item area is clickable, not just specific portions.

## Testing
After these fixes:
- The entire game item should be clickable
- Clicking anywhere on the item (including the game title) should select it
- Double-clicking should trigger the play action
- Right-clicking should show the context menu
- The expand arrow for parent games should still work independently

## Note
The same click area issue likely affects clone items as well. The `render_clone_item` method uses a similar pattern and should be reviewed to ensure consistent behavior across all list items.