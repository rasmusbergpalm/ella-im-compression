# Ella image compression

A tiny image compression library using arithmetic coding

### Usage

Encoding

`ella e input.png output.ela`

Decoding

`ella d input.ela output.png`

### Details

Filters the image by subtracting the predicted pixel values, given the already seen pixels. This reduces the image to values close to zero (the errors).

These errors are then compressed using [Arithmetic coding](https://en.wikipedia.org/wiki/Arithmetic_coding) with an adaptive distribution over symbols, that start out as uniform and update as symbols are encoded/decoded. 
This gets the size of the file quite close to the limit given by the entropy of the error distribution, which is again defined by how good the model is at predicting pixel values.

### Format

The file format is 

|bytes|description| dtype | example hex | decoded |
|---|---|---|---| --- |
| 0-3 | magic bytes| utf8 str| `454c 4c41` | ELLA |
| 4-7 | version | uint32| `0000 0001` | 1 | 
| 8-11 | width | uint32| `0000 0200` | 512 | 
| 12-15 | height | uint32| `0000 0200` | 512 |
| 16 | channels | uint8 | `03` | 3 |
| 17- | pixels | encoded | ... | ... |


