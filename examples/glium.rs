extern crate camera_capture;
#[macro_use] extern crate failure;
#[macro_use] extern crate glium;
extern crate image;


use std::time::Duration;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use failure::{Error, SyncFailure};
use glium::texture::{CompressedSrgbTexture2d, RawImage2d};
use glium::backend::Facade;
use glium::{glutin, Display, IndexBuffer, Program, Surface, VertexBuffer};
use glium::glutin::{EventsLoop, WindowBuilder, ContextBuilder};
use image::{ImageBuffer, Rgb};


fn main() {
    if let Err(e) = run() {
        eprintln!("An error occured: {}", e);
        for cause in e.iter_chain().skip(1) {
            eprintln!("... caused by: {}", cause);
        }
    }
}

fn run() -> Result<(), Error> {
    // Building the display. This object is the main object, containing the
    // OpenGL facade and everything else.
    let (mut events_loop, display) = create_display()?;

    // This will start a new thread which pulls images from the webcam and
    // sends them into a channel. We get the receiving end of the channel,
    // called `webcam_frames`.
    let (webcam_frames, webcam_thread) = start_webcam()?;

    // Build buffers for a fullscreen quad.
    let (vertex_buffer, index_buffer) = quad_buffers(&display)?;

    // Create a simple program to draw an image to the fullscreen quad
    let program = create_program(&display)?;

    let mut stop = false;
    while !stop {
        // We only wait for a new webcam image until some timeout is reached.
        let timeout = Duration::from_millis(500);
        if let Ok(frame) = webcam_frames.recv_timeout(timeout) {
            let (frame_width, frame_height) = frame.dimensions();

            // Take the frame from the webcam and convert it to a texture.
            let image = RawImage2d::from_raw_rgb_reversed(
                &frame,
                (frame_width, frame_height),
            );
            let texture = CompressedSrgbTexture2d::new(&display, image)?;

            // Pass the texture to the shader via uniforms.
            let uniforms = uniform! { tex: &texture };

            // Finally, draw the image on the screen
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 0.0);
            target.draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default()
            )?;
            target.finish()?;

            // Polling and handling the events received by the window.
            events_loop.poll_events(|event| {
                use glutin::{ElementState, VirtualKeyCode};

                // We are only interested in window events.
                let event = match event {
                    glutin::Event::WindowEvent { event, .. } => event,
                    _ => return,
                };

                match event {
                    // Stop the application when the close-button is clicked or
                    // ESC is pressed
                    glutin::WindowEvent::CloseRequested |
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => stop = true,
                    _ => {}
                }
            });
        } else {
            bail!("Webcam thread was killed or did not responded for {:?}. Stopping.", timeout);
        }
    }

    // After stopping the main program, we want to gracefully stop the thread
    // pulling in the webcam images.
    stop_webcam(webcam_frames, webcam_thread);

    Ok(())
}


// ===========================================================================
// ===== Webcam helper functions
// ===========================================================================
pub type CamFrame = ImageBuffer<Rgb<u8>, camera_capture::Frame>;

pub fn start_webcam() -> Result<(Receiver<CamFrame>, JoinHandle<()>), Error> {
    let (sender, receiver) = mpsc::channel();
    let cam = camera_capture::create(0).unwrap()
        .fps(30.0)
        .unwrap()
        .start()?;

    let webcam_thread = thread::spawn(move || {
        for frame in cam {
            if let Err(_) = sender.send(frame) {
                break;
            }
        }
    });

    Ok((receiver, webcam_thread))
}

pub fn stop_webcam(receiver: Receiver<CamFrame>, thread: JoinHandle<()>) {
    // We close our channel which will cause the other thread to stop itself.
    // The main thread then just waits for this to happen.
    drop(receiver);
    if let Err(_) = thread.join() {
        eprintln!("The webcam thread panicked before we tried to join it...");
    }
}


// ===========================================================================
// ===== OpenGL helper functions
// ===========================================================================

/// Creates the OpenGL context.
pub fn create_display() -> Result<(EventsLoop, Display), Error> {
    // Create the event loop for the window
    let events_loop = EventsLoop::new();

    // Configure the window
    let window = WindowBuilder::new()
        .with_title("camera_capture example");        ;

    // Configure the OpenGL context
    let context = ContextBuilder::new();

    // Put all together and create a finished "display"
    glium::Display::new(window, context, &events_loop)
        .map_err(|e| SyncFailure::new(e).into())
        .map(|context| (events_loop, context))
}

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

/// Creates and returns a vertex-/index-buffer pair which represents a full
/// screen quad (we don't want to do any fancy 3D drawing, just simply draw
/// a texture on the whole screen, that is: the part inside of our window).
pub fn quad_buffers<F: Facade>(
    display: &F
) -> Result<(VertexBuffer<Vertex>, IndexBuffer<u16>), Error> {
    use glium::index::PrimitiveType;

    // The vertex buffer simply contains the OpenGL screen coordinates. That
    // way we can simply draw on the full screen.
    let vertex_data = [
        Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
        Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] },
    ];

    // Create both OpenGL buffers
    let vertex_buffer = VertexBuffer::new(
        display,
        &vertex_data,
    )?;
    let index_buffer = IndexBuffer::new(
        display,
        PrimitiveType::TriangleStrip,
        &[1 as u16, 2, 0, 3],
    )?;

    Ok((vertex_buffer, index_buffer))
}

/// Creates a simple shader program which simply renders a texture `tex` with
/// the texture coordinates `tex_coords`.
pub fn create_program<F: Facade>(display: &F) -> Result<Program, Error> {
    // Compiling shaders and linking them together
    program!(display,
        140 => {
            vertex: "
                #version 140

                in vec2 position;
                in vec2 tex_coords;

                out vec2 v_tex_coords;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;

                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
        },
    ).map_err(|e| e.into())
}
