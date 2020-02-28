# Pokemon Gen 3 Clone

See current progress at 

https://raw.githubusercontent.com/dwbrite/pkmn-rs/master/peek.webm

## Setup

Install rust+cargo (via rustup), vulkan sdk(?), and gtk-dev(?)

Renderdoc is useful for seeing how drawing works.

### Linux + Windows
`cargo run`

### MacOS
Pseudo-WIP. Make a PR for it if you care.
It's really just getting MoltenVK working with this, but I don't have a Mac so I couldn't test it if I wanted to.

## Rendering

Start with the basic vulkan initialization:

- get the required window extensions
- create a vulkan instance
- get a physical graphics device
- get the device's queue

then initialize everything for drawing to the screen

- create a window
- create a surface on the window
- create a swapchain with image buffers for the surface
  - PresentMode: FIFO. The game is light enough that we should always be able to run 60FPS...?
  - TODO: Use Relaxed if available, or create a setting for FIFO vs
- create a viewport for rendering

rendering loop:

- wait for cleanup from the previous frame to finish
- recreate swapchain if necessary
- create and execute a command buffer for the frame
- handle input (this shouldn't be in the render loop)

### Frame rendering
