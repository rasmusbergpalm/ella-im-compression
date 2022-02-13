# Ella image compression

A tiny image compression library using arithmetic coding

### Usage

Encoding

`ella e <input> <output> <model>` e.g. `ella e input.png output.ella 1`

Decoding

`ella d <input> <output>` e.g. `ella d input.ella output.png`

### Details

A model predicts pixel values given the already observed pixel values which result in a distribution of prediction errors.
These errors are then compressed using [Arithmetic coding](https://en.wikipedia.org/wiki/Arithmetic_coding) with an adaptive distribution over symbols, that start out as uniform and update as symbols are encoded/decoded. 
This gets the size of the file quite close to the limit given by the entropy of the error distribution, which is again defined by how good the model is at predicting pixel values.

### Models
The format supports multiple different models for predicting pixel values. The following models are supported

0. **LEFT** The pixels are decoded from top to bottom, left to right, and the predicted pixel value is the last pixel value decoded.
1. **AVG** The pixels are decoded from top to bottom, left to right, and the predicted pixel value is the average (rounded down) of the pixel to the left, above and diagonal up left. This is similar to the model used in PNG. 

### Format

The file format is 

| bytes | description | dtype    | example hex | decoded |
|-------|-------------|----------|-------------|---------|
| 0-3   | magic bytes | utf8 str | `454c 4c41` | ELLA    |
| 4-7   | version     | uint32   | `0000 0001` | 1       | 
| 8-11  | width       | uint32   | `0000 0200` | 512     | 
| 12-15 | height      | uint32   | `0000 0200` | 512     |
| 16    | channels    | uint8    | `03`        | 3       |
| 17    | model       | uint16   | `0001`      | 1       |
| 18-   | pixels      | encoded  | ...         | ...     |


