// Liam Ballard
// COP3504
// Project 3

use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::fmt;

// Struct to represent TGA header data
#[derive(PartialEq, Debug)]
#[derive(Clone)]
struct Header {
    id_length: u8,
    type_map: u8,
    image_type: u8,
    color_map_details: [u8; 5],
    image_details: [u8; 10],
}

// Converts TGA header instance to byte array
impl Header {
    fn to_bytes(&self) -> [u8; 18] {
        let mut bytes = [0u8; 18];
        bytes[0] = self.id_length;
        bytes[1] = self.type_map;
        bytes[2] = self.image_type;
        bytes[3..8].copy_from_slice(&self.color_map_details);
        bytes[8..18].copy_from_slice(&self.image_details);
        bytes
    }
}

// Struct that represents pixel data
struct Pixels {
    data: Vec<Pixel>,
}

// Struct that represents data (header and pixels)
struct Data {
    header: Header,
    pixels: Pixels,
}

// Struct representing a single BGR pixel
#[derive(PartialEq, Debug)]
#[derive(Clone)]
struct Pixel {
    blue: u8,
    green: u8,
    red: u8,
}

// Loads a TGA file and returns TGAData
fn read_tga(file_path: &str) -> Result<Data, io::Error> {
    let mut data = Vec::new(); // Stores binary TGA image data
    let mut file = fs::File::open(file_path)?; // Opens file

    // Read TGA header
    file.read_to_end(&mut data)?;

    // Split data into header and pixels
    let header = &data[0..18];
    let pixels = &data[18..];
    // Make header data
    let header = Header {
        id_length: header[0],
        type_map: header[1],
        image_type: header[2],
        color_map_details: [header[3], header[4], header[5], header[6], header[7]],
        image_details: 
        [
            header[8], header[9], header[10], header[11], header[12], header[13],
            header[14], header[15], header[16], header[17],
        ],
    };

    // Place pixel data into BGR pixels
    let pixels = make_pixels(pixels);

    // Creates Data struct w/ header and pixel data
    Ok(Data {header, pixels: Pixels {data: pixels}, })
}

// Turns pixel data into BGR pixels
fn make_pixels(pixel_data: &[u8]) -> Vec<Pixel> {
    let mut bgr_pixels = Vec::new(); // Holds pixel data
    let mut index = 0;
    while index < pixel_data.len() {
        // Ensure that there are at least 3 bytes remaining in the slice
        if index + 2 < pixel_data.len() {
            // Add each color to Pixel vector
            let blue = pixel_data[index];
            let green = pixel_data[index + 1];
            let red = pixel_data[index + 2];
            bgr_pixels.push(Pixel {blue, green, red});
        }
        // Move to next BGR pixel
        index += 3;
    }
    bgr_pixels
}

// Function that writes a new TGA file
fn write_tga(file_path: &str, data: Data) -> Result<(), io::Error> {
    // Attempt to create a new file at specified path
    let file_result = fs::File::create(file_path);

    // Check if file creation was successful
    match file_result {
        Ok(mut file) => {
            // Create a vector to store pixel data
            let mut pixel_data = Vec::new();
            for pixel in &data.pixels.data {
                // Append blue, green, and red components to pixel data vector
                pixel_data.push(pixel.blue);
                pixel_data.push(pixel.green);
                pixel_data.push(pixel.red);
            }

            // Attempt to write TGA header to file
            let header_write_result = file.write_all(&data.header.to_bytes());

            // Check if writing header was successful
            if let Err(e) = header_write_result {
                return Err(e); // Return an error if writing the header fails
            }

            // Attempt to write pixel data to file
            let pixel_data_write_result = file.write_all(&pixel_data);

            // Check if writing pixel data was successful
            if let Err(e) = pixel_data_write_result {
                return Err(e); // Return an error if writing the pixel data fails
            }

            Ok(()) // Return success if all operations are successful
        }
        Err(e) => Err(e), // Return error if file cannot be created
    }
}

// Multiply blending operation
fn multiply_blend(top_pixels: &Pixels, bottom_pixels: &Pixels) -> Vec<Pixel> {
    // Ensure both layers have the same dimensions
    if top_pixels.data.len() != bottom_pixels.data.len() {
        panic!("Layers have different dimensions.");
    }
    let mut multiplied_pixels = Vec::new(); // Holds new pixel data

    // Get reference to the pixel data for both layers
    let top_data = &top_pixels.data;
    let bottom_data = &bottom_pixels.data;

    for i in 0..top_data.len() {
        let top_pixel = &top_data[i];
        let bottom_pixel = &bottom_data[i];

        // Multiply color channels of the two pixels (takes care of 0 <= value <= 255 constraint)
        let blue = (top_pixel.blue as f32 * bottom_pixel.blue as f32 / 255.0).round() as u8;
        let green = (top_pixel.green as f32 * bottom_pixel.green as f32 / 255.0).round() as u8;
        let red = (top_pixel.red as f32 * bottom_pixel.red as f32 / 255.0).round() as u8;

        // Create new pixel with result
        let modified_pixel = Pixel {blue, green, red};

        // Add the new pixel to the multiplied_pixels vector
        multiplied_pixels.push(modified_pixel);
    }

    multiplied_pixels
}

// Implements Subtract blending mode
// Top layer is subtracted FROM the bottom layer
// Currently using Multiply blending logic
fn subtract_blend(top_pixels: &Pixels, bottom_pixels: &Pixels) -> Vec<Pixel> {
    // Ensure both layers have the same dimensions
    if top_pixels.data.len() != bottom_pixels.data.len() {
        panic!("Layers have different dimensions.");
    }

    let mut subtracted_pixels = Vec::new();

    // Get reference to the pixel data for both layers
    let top_data = &top_pixels.data;
    let bottom_data = &bottom_pixels.data;

    for i in 0..top_data.len() {
        let top_pixel = &top_data[i];
        let bottom_pixel = &bottom_data[i];

        // Subtract top layer pixels from bottom layer pixels (takes care of 0 <= value <= 255 constraint)
        let blue = bottom_pixel.blue.saturating_sub(top_pixel.blue);
        let green = bottom_pixel.green.saturating_sub(top_pixel.green);
        let red = bottom_pixel.red.saturating_sub(top_pixel.red);

        // Create a new pixel with the result
        let modified_pixel = Pixel { blue, green, red };

        // Add the new pixel to the subtracted_pixels vector
        subtracted_pixels.push(modified_pixel);
    }

    subtracted_pixels
}

// Implements screen blending mode
fn screen_blend(top_pixels: &Pixels, bottom_pixels: &Pixels) -> Vec<Pixel> {
    // Ensure both layers have the same dimensions
    if top_pixels.data.len() != bottom_pixels.data.len() {
        panic!("Layers have different dimensions");
    }
    let mut screen_pixels = Vec::new();
    // Get a reference to the pixel data for both layers
    let top_data = &top_pixels.data;
    let bottom_data = &bottom_pixels.data;

    // For each pixel
    for i in 0..top_data.len() {
        let top_pixel = &top_data[i];
        let bottom_pixel = &bottom_data[i];

        // Find inverted values for blue channel
        let inverted_top_blue = 255 - top_pixel.blue;
        let inverted_bottom_blue = 255 - bottom_pixel.blue;
        let blue = 255 - ((inverted_top_blue as f32* inverted_bottom_blue as f32 / 255.0).round() as u8);

        // Find inverted values for green channel
        let inverted_top_green = 255 - top_pixel.green;
        let inverted_bottom_green = 255 - bottom_pixel.green;
        let green = 255 - ((inverted_top_green as f32 * inverted_bottom_green as f32 / 255.0).round() as u8);

        // Find inverted values for red channel
        let inverted_top_red = 255 - top_pixel.red;
        let inverted_bottom_red = 255 - bottom_pixel.red;
        let red = 255 - ((inverted_top_red as f32 * inverted_bottom_red as f32 / 255.0).round() as u8);

        // Create a new pixel with the result
        let screen_pixel = Pixel {blue, green, red};

        // Add modified pixel to the screen_pixels vector
        screen_pixels.push(screen_pixel);
    }

    screen_pixels
}

// Overlay blending mode
fn overlay_blend(top_pixels: &Pixels, bottom_pixels: &Pixels) -> Vec<Pixel> {
    // Ensure both layers have same dimensions
    if top_pixels.data.len() != bottom_pixels.data.len() {
        panic!("Layers don't have same dimensions");
    }

    let mut overlay_pixels = Vec::new(); // Holds overlay pixel data

    // Get reference to the pixel data for both layers
    let top_data = &top_pixels.data;
    let bottom_data = &bottom_pixels.data;

    // For each top and bottom pixel
    for i in 0..top_data.len() {
        let top_pixel = &top_data[i];
        let bottom_pixel = &bottom_data[i];

        let red;
        let blue;
        let green;

        // Check if bottom layer pixel intensity is less than 128
        // If true, use the formula for the Multiply blending mode
        if bottom_pixel.blue < 128 {
            blue = (2.0 * top_pixel.blue as f32 * bottom_pixel.blue as f32 / 255.0).round() as u8;
        }
        else {
            blue = (255.0 - 2.0 * (255.0 - top_pixel.blue as f32) * (255.0 - bottom_pixel.blue as f32) / 255.0).round() as u8;
        }
        if bottom_pixel.green < 128 {
            green = (2.0 * top_pixel.green as f32 * bottom_pixel.green as f32 / 255.0).round() as u8;
        }
        else {
            green = (255.0 - 2.0 * (255.0 - top_pixel.green as f32) * (255.0 - bottom_pixel.green as f32) / 255.0).round() as u8;
        }
        if bottom_pixel.red < 128 {
            red = (2.0 * top_pixel.red as f32 * bottom_pixel.red as f32 / 255.0).round() as u8;
        }
        else {
             red = (255.0 - 2.0 * (255.0 - top_pixel.red as f32) * (255.0 - bottom_pixel.red as f32) / 255.0).round() as u8;
        }

        // Create new pixel with result
        let new_pixel = Pixel {blue, green, red};

        // Add new pixel to overlay_pixels vector
        overlay_pixels.push(new_pixel);
    }

    overlay_pixels
}

// Combines image channels
fn combine_channels(blue_channel: &Data, green_channel: &Data, red_channel: &Data) -> Data{
    // Ensure all layers have the same dimensions
    if blue_channel.pixels.data.len() != green_channel.pixels.data.len() || blue_channel.pixels.data.len() != red_channel.pixels.data.len() {
        panic!("Channels don't have same dimensions");
    }

    let mut combined_pixels = Vec::new(); // Vector storing new pixels

    // For each pixel
    for i in 0..blue_channel.pixels.data.len() {

        // Create new pixel with combined channels
        let combined_channels = Pixel {
            blue: blue_channel.pixels.data[i].blue,
            green: green_channel.pixels.data[i].green,
            red: red_channel.pixels.data[i].red,
        };

        // Add new pixel to combined_pixels vector
        combined_pixels.push(combined_channels);
    }

    // New image, with old image header and new pixel data
    let new_image = Data {
        header: blue_channel.header.clone(),
        pixels: Pixels {data: combined_pixels}
    };

    new_image
}

fn flip(image: &Data) -> Vec<Pixel> {
    let mut flipped_pixels = Vec::new(); // Vector to store flipped pixel data

    // Extract width and height from header
    let width = image.header.image_details[0] as usize;
    let height = image.header.image_details[1] as usize;

    // Loop through each row of the original image
    for i in 0..height {
        // Loop through each column of the original image
        for j in 0..width {
            // Calculate index for flipped image
            let index = (height - 1 - i) * width + j;
             // Append pixel at desired index to flipped pixel vector
            flipped_pixels.push(image.pixels.data[index].clone());
        }
    }

    flipped_pixels
}

// Combine images
fn combine_images(images: [&Data; 4]) -> Data {
    // Make sure images have same dimensions
    let width = images[0].header.image_details[8] as usize;
    let height = images[0].header.image_details[9] as usize;

    for i in 1..4 {
        if images[i].header.image_details[8] as usize != width || images[i].header.image_details[9] as usize != height {
            panic!("Images have different dimensions");
        }
    }

    let mut combined_pixels = Vec::new();  // New vector to store combined pixel data

    // Loop through each pixel and combine quadrants
    for i in 0..height {
        for j in 0..width {
            let quadrant_index = if j < width / 2 {
                if i < height / 2 {
                    0 // Top-left
                } else {
                    2 // Bottom-left
                }
            } else {
                if i < height / 2 {
                    1 // Top-right
                } else {
                    3 // Bottom-right
                }
            };

            let pixel = &images[quadrant_index].pixels.data[i * width + j];

            // Add the combined pixel to the vector
            combined_pixels.push(pixel.clone());
        }
    }

    // New header, reflects image dimension changes
    let mut combined_header = Header {
        id_length: 0,
        type_map: 0,
        image_type: 2,
        color_map_details: [0; 5],
        image_details: [0; 10],
    };

    combined_header.image_details[5] = 4;
    combined_header.image_details[6] = 0;
    combined_header.image_details[7] = 4;
    combined_header.image_details[8] = 24;

    // New Data struct for combined image
    let combined_image = Data {
        header: combined_header,
        pixels: Pixels {data: combined_pixels},
    };

    combined_image
}

// Part 1 function
// Use Multiply blending mode to combine “layer1.tga” (top layer) with “pattern1.tga” (bottom)
fn part1() {
    // Load the top layer TGA file (layer1.tga).
    let top_layer = read_tga("input/layer1.tga").expect("Failed to load top layer TGA file");

    // Load the bottom layer TGA file (pattern1.tga).
    let bottom_layer = read_tga("input/pattern1.tga").expect("Failed to load bottom layer TGA file");

    // Ensure both layers have same dimensions
    if top_layer.pixels.data.len() != bottom_layer.pixels.data.len() {
        panic!("Layers have different dimensions.");
    }

    // Multiply layers
    let blended_pixels = multiply_blend(&top_layer.pixels, &bottom_layer.pixels);

    // Data instance result
    let result_data = Data {
        header: top_layer.header, // Uses header from top layer
        pixels: Pixels { data: blended_pixels },
    };

    // Save result as "part1.tga" in output folder
    write_tga("output/part1.tga", result_data).expect("Failed to save the result");
}

// Part 2 function
// Use the Subtract blending mode to combine “layer2.tga” (top layer) with “car.tga” (bottom layer)
// This mode subtracts the top layer from the bottom layer
fn part2() {
    // Load top layer TGA file (layer2.tga)
    let top_layer = read_tga("input/layer2.tga").expect("Failed to load top layer");

    // Load bottom layer TGA file (car.tga)
    let bottom_layer = read_tga("input/car.tga").expect("Failed to load bottom layer");

    // Ensure both layers have the same dimensions
    if top_layer.pixels.data.len() != bottom_layer.pixels.data.len() {
        panic!("Layers don't have same dimensions");
    }

    // Perform the Subtract blending operation on the two layers.
    let blended_pixels = subtract_blend(&top_layer.pixels, &bottom_layer.pixels);

    // Create TGAData instance for the result
    let result_data = Data {
        header: top_layer.header, // Uses header from top layer
        pixels: Pixels {data: blended_pixels},
    };

    // Save result as "part2.tga" in output folder
    write_tga("output/part2.tga", result_data).expect("Failed to save the result as TGA.");
}

// Part 3 function
// Use the Multiply blending mode to combine “layer1.tga” with “pattern2.tga”, and store the
// results temporarily. Load the image “text.tga” and, using that as the top layer, combine it with
// the previous results of layer1/pattern2 using the Screen blending mode
fn part3() {
    // Multiply
    // Load the top layer TGA file (layer1.tga)
    let top_layer = read_tga("input/layer1.tga").expect("Failed to load top layer");

    // Load the bottom layer TGA file (pattern2.tga)
    let bottom_layer = read_tga("input/pattern2.tga").expect("Failed to load bottom layer");

    // Make sure both layers have same dimensions
    if top_layer.pixels.data.len() != bottom_layer.pixels.data.len() {
        panic!("Layers don't have same dimensions");
    }

    // Multiply blending
    let blended_pixels = multiply_blend(&top_layer.pixels, &bottom_layer.pixels);

    // Create TGAData instance for Multiply result
    let multiply_result = Data {
        header: top_layer.header, // Uses header from top layer
        pixels: Pixels {data: blended_pixels},
    };

    // Load top layer TGA file (text.tga)
    let top_layer = read_tga("input/text.tga").expect("Failed to load top layer");

    // Ensure both layers have the same dimensions
    if top_layer.pixels.data.len() != multiply_result.pixels.data.len() {
        panic!("Layers don't have same dimensions");
    }

    // Screen blending
    let blended_pixels = screen_blend(&top_layer.pixels, &multiply_result.pixels);

    // Create TGAData instance for Screen result
    let output_data = Data {
        header: top_layer.header, // Uses header from the top layer
        pixels: Pixels {data: blended_pixels},
    };

    // Save final result in the output folder
    write_tga("output/part3.tga", output_data).expect("Failed to save the result as TGA.");
}


// Part 4 function
//Multiply “layer2.tga” with “circles.tga”, and store it. Load “pattern2.tga” and, using that as the
// top layer, combine it with the previous result using the Subtract blending mode
fn part4() {
    // Multiply
    // Load top layer TGA file (layer2.tga)
    let top_layer = read_tga("input/layer2.tga").expect("Failed to load top layer");

    // Load bottom layer TGA file (circles.tga)
    let bottom_layer = read_tga("input/circles.tga").expect("Failed to load bottom layer");

    // Make sure both layers have the same dimensions
    if top_layer.pixels.data.len() != bottom_layer.pixels.data.len() {
        panic!("Layers don't have same dimensions");
    }

    // Multiply blending
    let blended_pixels = multiply_blend(&top_layer.pixels, &bottom_layer.pixels);

    // Create TGAData instance for result
    let result_data = Data {
        header: top_layer.header, // Uses header from the top layer
        pixels: Pixels { data: blended_pixels },
    };

    // Subtract
    // Load top layer TGA file (pattern2.tga)
    let top_layer = read_tga("input/pattern2.tga").expect("Failed to load top layer");

    // Set bottom layer to result_data
    let bottom_layer = result_data;

    // Ensure both layers have same dimensions
    if top_layer.pixels.data.len() != bottom_layer.pixels.data.len() {
        panic!("Layers have different dimensions");
    }

    // Subtract blending
    let blended_pixels = subtract_blend(&top_layer.pixels, &bottom_layer.pixels);

    // Create TGAData instance for result
    let output_data = Data {
        header: top_layer.header, // Uses header from the top layer
        pixels: Pixels {data: blended_pixels},
    };

    // Save result in output folder
    write_tga("output/part4.tga", output_data).expect("Failed to save the result as TGA file");
}

// Part 5 function
// Combine “layer1.tga” (as the top layer) with “pattern1.tga” using the Overlay blending mode
fn part5() {
    // Load the top layer TGA file (layer1.tga).
    let top_layer = read_tga("input/layer1.tga").expect("Failed to load top layer");

    // Load the bottom layer TGA file (pattern1.tga).
    let bottom_layer = read_tga("input/pattern1.tga").expect("Failed to load bottom layer");

    // Ensure both layers have the same dimensions
    if top_layer.pixels.data.len() != bottom_layer.pixels.data.len() {
        panic!("Layers have different dimensions");
    }

    // Multiply the layers
    let blended_pixels = overlay_blend(&top_layer.pixels, &bottom_layer.pixels);

    // Create TGAData instance for result
    let result_data = Data {
        header: top_layer.header, // Uses header from the top layer
        pixels: Pixels {data: blended_pixels},
    };

    // Save result as "part1.tga" in output folder
    write_tga("output/part5.tga", result_data).expect("Failed to save result");
}

// Part 6 function
// Load “car.tga” and add 200 to the green channel
fn part6() {
    // Load car.tga
    let mut image_to_modify = read_tga("input/car.tga").expect("Failed to load car.tga");

    // Loop through pixels in car image, adding 200 to green channel
    for pixel in &mut image_to_modify.pixels.data {
        let modified_green = pixel.green.saturating_add(200); // Saturating add ensures result remains <255.
        pixel.green = modified_green;
    }

    // Save modified image
    write_tga("output/part6.tga", image_to_modify).expect("Failed to save image.");
}

// Part 7 function
// Load “car.tga” and scale (multiply) the red channel by 4, and the blue channel by 0. This will
// increase the intensity of any red in the image, while negating any blue it may have
fn part7() {
    // Load car.tga
    let mut image_to_modify = read_tga("input/car.tga").expect("Failed to load car.tga.");

    // Loop through pixels in car image, multiplying red channel by 4 and negating blue channel
    for pixel in &mut image_to_modify.pixels.data {
        let modified_blue = 0; // Blue is negated
        pixel.blue = modified_blue;
        let modified_red = pixel.red.saturating_mul(4); // Multiply red channel by 4, saturating ensures result <255
        pixel.red = modified_red;
    }

    // Save modified image
    write_tga("output/part7.tga", image_to_modify).expect("Failed to save image");
}

// Part 8 function
// Load “car.tga” and write each channel to a separate file: the red channel should be “part8_r.tga”,
// the green channel should be “part8_g.tga”, and the blue channel should be “part8_b.tga”
fn part8() {
    let image = read_tga("input/car.tga").expect("Failed to load car.tga");

    // Loop through pixels in car image, extracting red channel
    let mut red_channel_pixels = Vec::new();

    for i in 0..image.pixels.data.len() {
        let current_pixel = &image.pixels.data[i]; 

        // Extract red channel, set blue and green to 0
        let blue = current_pixel.red;
        let green = current_pixel.red;
        let red = current_pixel.red;

        // Create new pixel with the extracted red channel
        let modified_pixel = Pixel {blue, green, red};

        // Add new pixel to red_channel_pixels vector
        red_channel_pixels.push(modified_pixel);
    }

    let result_data = Data {
        header: image.header.clone(), // Use clone to avoid moving header
        pixels: Pixels { data: red_channel_pixels },
    };

    write_tga("output/part8_r.tga", result_data).expect("Failed to save image.");

    // Handle green channel
    let mut green_channel_pixels = Vec::new();

    for i in 0..image.pixels.data.len() {
        let current_pixel = &image.pixels.data[i];

        // Green channel, set blue and green to 0
        let blue = current_pixel.green;
        let green = current_pixel.green;
        let red = current_pixel.green;

        // Create a new pixel with extracted red channel
        let modified_pixel = Pixel {blue, green, red};

        // Add the new pixel to red_channel_pixels vector
        green_channel_pixels.push(modified_pixel);
    }

    let result_data = Data {
        header: image.header.clone(),
        pixels: Pixels { data: green_channel_pixels },
    };

    write_tga("output/part8_g.tga", result_data).expect("Failed to save image.");

    // Blue channel
    let mut blue_channel_pixels = Vec::new();

    for i in 0..image.pixels.data.len() {
        let current_pixel = &image.pixels.data[i]; 

        let blue = current_pixel.blue;
        let green = current_pixel.blue;
        let red = current_pixel.blue;

        // Create a new pixel with the extracted blue channel
        let modified_pixel = Pixel {blue, green, red};

        // Add the new pixel to the blue_channel_pixels vector
        blue_channel_pixels.push(modified_pixel);
    }

    let result_data = Data {
        header: image.header.clone(), // Use clone to avoid moving header
        pixels: Pixels { data: blue_channel_pixels },
    };

    write_tga("output/part8_b.tga", result_data).expect("Failed to save image");

}    

// Part 9 function
// Load “layer_red.tga”, “layer_green.tga” and “layer_blue.tga”, and combine the three files into
// one file. The data from “layer_red.tga” is the red channel of the new image, layer_green is
// green, and layer_blue is blue
fn part9() {
    // Load the three layers
    let blue_channel = read_tga("input/layer_blue.tga").expect("Failed to load layer_blue.tga");
    let green_channel = read_tga("input/layer_green.tga").expect("Failed to load layer_green.tga");
    let red_channel = read_tga("input/layer_red.tga").expect("Failed to load layer_red.tga");

    // Combine the layers and save the result
    let new_image = combine_channels(&blue_channel, &green_channel, &red_channel);

    write_tga("output/part9.tga", new_image).expect("Failed to write TGA file");
}

// Part 10 function 
// Load “text2.tga”, and rotate it 180 degrees, flipping it upside down
fn part10() {
    // Load text2.tga
    let image_to_modify = read_tga("input/text2.tga").expect("Failed to load car.tga");
    let flipped_pixels = flip(&image_to_modify);
    let flipped_image_data = Data {
        header: image_to_modify.header.clone(), // Uses header from text2.tga
        pixels: Pixels {data: flipped_pixels}, // Flipped pixels
    };
    write_tga("output/part10.tga", flipped_image_data).expect("Unable to flip image data");
}

// Extra credit function
// Create a new file that is the combination of car.tga, circles.tga, pattern1.tga, and text.tga
fn extra_credit() {
    // Load individual images
    let car_image = read_tga("input/car.tga").expect("Failed to load car.tga");
    let circles_image = read_tga("input/circles.tga").expect("Failed to load circles.tga");
    let pattern1_image = read_tga("input/pattern1.tga").expect("Failed to load pattern1.tga");
    let text_image = read_tga("input/text.tga").expect("Failed to load text.tga");

    // Use combined image method
    let images = [&car_image, &circles_image, &pattern1_image, &text_image];
    let combined_image = combine_images(images);

    // Write the combined image to output/extracredit.tga
    write_tga("output/extracredit.tga", combined_image).expect("Failed to write extracredit.tga");
}

// Test format for pixels 
impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pixel(R: {}, G: {}, B: {})", self.red, self.green, self.blue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_part1() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part1.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part1.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
        
    }

    #[test]
    fn test_part2() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part2.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part2.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
        
    }

    #[test]
    fn test_part3() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part3.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part3.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
        
    }

    #[test]
    fn test_part4() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part4.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part4.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
        
    }

    #[test]
    fn test_part5() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part5.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part5.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_part6() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part6.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part6.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_part7() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part7.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part7.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_part8_red() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part8_r.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part8_r.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_part8_green() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part8_g.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part8_g.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_part8_blue() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part8_b.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part8_b.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_part9() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part9.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part9.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_part10() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/part10.tga").expect("Failed to load actual result TGA file.");
        
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_part10.tga").expect("Failed to load expected result TGA file.");
        
        // Compare headers to ensure they match
        for (actual_pixel, expected_pixel) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()) {
            if actual_pixel != expected_pixel {
                println!("Pixels don't match: Actual: {}, Expected: {}", actual_pixel, expected_pixel);
            }
        }        

        // Compare BGR pixel data to ensure they match
        for (actual_value, expected_value) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()) {
            if actual_value != expected_value {
                println!("Header bytes don't match: Actual: {}, Expected: {}", actual_value, expected_value);
            }
        }
    }

    #[test]
    fn test_extra_credit() {
        // Load the actual result produced by your code
        let actual_result = read_tga("output/extracredit.tga").expect("Failed to load actual result TGA file.");
    
        // Load the expected result from the example file
        let expected_result = read_tga("examples/EXAMPLE_extracredit.tga").expect("Failed to load expected result TGA file.");
    
        // Compare headers to ensure they match
        for (i, (actual_byte, expected_byte)) in actual_result.header.to_bytes().iter().zip(expected_result.header.to_bytes().iter()).enumerate() {
            if actual_byte != expected_byte {
                println!("Byte {} doesn't match: Actual: {}, Expected: {}", i, actual_byte, expected_byte);
            }
        }
    
        // Compare BGR pixel data to ensure they match
        for (i, (actual_pixel, expected_pixel)) in actual_result.pixels.data.iter().zip(expected_result.pixels.data.iter()).enumerate() {
            if actual_pixel != expected_pixel {
                println!("Pixel {} doesn't match: Actual: {}, Expected: {}", i, actual_pixel, expected_pixel);
            }
        }
    }        
}

fn main() {
    part1();
    part2();
    part3();
    part4();
    part5();
    part6();
    part7();
    part8();
    part9();
    part10();
    extra_credit();
}
