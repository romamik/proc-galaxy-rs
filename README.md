# Procedural galaxy in Rust
## About the project

This should be procedural generated galaxy. Main purpose of this project is to teach myself rust.

Final result should be a program that after running displays a 2d galaxy and allows user to pan and zoom in and out. This galaxy should be procedurally generated. There should be a lot of stars and user should be able to zoom in to any star to view it in detail.

I would be cool if there will be more than one galaxy. 

## Day 1. 22/05/2022
### Current state

https://github.com/romamik/proc-galaxy-rs/commit/172c834faef36c738422fe1a32da358f1cb213b5

I've spent around 4 hours on this.

I've decided that main building block of space will entity called `block`. That's a square rectangle, that is divided in sub-blocks. Each sub-block has it's own sub-blocks. Whole space is represented with root block. 

At each time program render some block, lets call it `current block`, and it's subblocks behind. If the user zooms in one the subblocks is selected as current and from this point program renders this new current block and it's subblocks. If the user zoom's out parent block is selected as current.

Each block can be addressed as list of block coordinates:
* [] - empty list for root block
* [(1,1)] - child of root block at coordinates 1,1
* [(1,1), (2,2)] - child of prev.block at coordinates 2,2
* etc.

Camera position is specified with block address, coordinates inside block `0.0..1.0`, and zoom level. Zoom level 0 means that current block occupies whole screen, zoom level 1 means that child block of the current block occupies whole screen.

Rendering is very basic: just current block and it's subblocks rectangles plus text with current camera position.

Controls also very basic: arrow keys for pan and 'z' and 'x' keys for zoom.

### What's next

- panning should be done in screen coordinates. I mean, visible panning speed should not depend on zoom level. That should be easy. 
- render all visible top-level blocks, not just the current. 
- separate code into modules. I've tried this, but got compiler errors. Need to read some docs.
- actual procedural generation of stars. Just random stars without galaxies as the first step. I need to figure out how things should be.

### Thoughts on procedural generation

Root block gets random seed.
Root block generates some stars, just coordinates, brightness, etc. And random seeds for each subblock. Let's call this level of detail level-0.
Subblocks get their random seeds and list of level-0 stars from parent block that belong to them. Each subblock generates level-1 details for that stars so this stars can be rendered in more details. And also generate new level-0 stars, so that we can zoom in further.

This idea can be easily expended to galaxies. At level-0 galaxy is just like star: a point with coordinates. At level-1 it can have type (like spiral, elliptical) and other parameters like number of arms for spiral galaxy. At this level it can be rendered as small texture. And at level-2 it can pass star density distribution parameters to subblocks so that they can generate actual stars.

## Day 1 22/05/2022 part 2

Spent one hour and half.
Minor improvements.

## Day 2 23/05/2022

Spent about an hour.
* Added block names and knowledge what block is being drawn, draw block names.
* Smooth movement using is_key_down instead of last_key_pressed.

Need to draw only subblocks on the screen, because there is huge lag when drawing 3 levels of blocks.