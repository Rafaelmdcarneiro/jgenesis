// Generated using the following Python one-liner:
// [[round(brightness / 15 * round(color * 255 / 31)) for color in range(32)] for brightness in range(16)]
pub const TABLE: &[[u8; 32]; 16] = &[
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    [
        0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 12, 12, 13, 13, 14, 14,
        15, 15, 16, 16, 17,
    ],
    [
        0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        27, 29, 30, 31, 32, 33, 34,
    ],
    [
        0, 2, 3, 5, 7, 8, 10, 12, 13, 15, 16, 18, 20, 21, 23, 25, 26, 28, 30, 31, 33, 35, 36, 38,
        39, 41, 43, 44, 46, 48, 49, 51,
    ],
    [
        0, 2, 4, 7, 9, 11, 13, 15, 18, 20, 22, 24, 26, 29, 31, 33, 35, 37, 39, 42, 44, 46, 48, 50,
        53, 55, 57, 59, 61, 64, 66, 68,
    ],
    [
        0, 3, 5, 8, 11, 14, 16, 19, 22, 25, 27, 30, 33, 36, 38, 41, 44, 47, 49, 52, 55, 58, 60, 63,
        66, 69, 71, 74, 77, 80, 82, 85,
    ],
    [
        0, 3, 6, 10, 13, 16, 20, 23, 26, 30, 33, 36, 40, 43, 46, 49, 53, 56, 59, 62, 66, 69, 72,
        76, 79, 82, 86, 89, 92, 96, 99, 102,
    ],
    [
        0, 4, 7, 12, 15, 19, 23, 27, 31, 35, 38, 42, 46, 50, 54, 57, 62, 65, 69, 73, 77, 81, 84,
        88, 92, 96, 100, 104, 107, 112, 115, 119,
    ],
    [
        0, 4, 9, 13, 18, 22, 26, 31, 35, 39, 44, 48, 53, 57, 61, 66, 70, 75, 79, 83, 88, 92, 97,
        101, 105, 110, 114, 118, 123, 127, 132, 136,
    ],
    [
        0, 5, 10, 15, 20, 25, 29, 35, 40, 44, 49, 54, 59, 64, 69, 74, 79, 84, 89, 94, 99, 104, 109,
        113, 118, 124, 128, 133, 138, 143, 148, 153,
    ],
    [
        0, 5, 11, 17, 22, 27, 33, 39, 44, 49, 55, 60, 66, 71, 77, 82, 88, 93, 99, 104, 110, 115,
        121, 126, 131, 137, 143, 148, 153, 159, 165, 170,
    ],
    [
        0, 6, 12, 18, 24, 30, 36, 43, 48, 54, 60, 66, 73, 78, 84, 90, 97, 103, 109, 114, 121, 127,
        133, 139, 144, 151, 157, 163, 169, 175, 181, 187,
    ],
    [
        0, 6, 13, 20, 26, 33, 39, 46, 53, 59, 66, 72, 79, 86, 92, 98, 106, 112, 118, 125, 132, 138,
        145, 151, 158, 165, 171, 178, 184, 191, 198, 204,
    ],
    [
        0, 7, 14, 22, 29, 36, 42, 50, 57, 64, 71, 78, 86, 93, 100, 107, 114, 121, 128, 135, 143,
        150, 157, 164, 171, 179, 185, 192, 199, 207, 214, 221,
    ],
    [
        0, 7, 15, 23, 31, 38, 46, 54, 62, 69, 77, 84, 92, 100, 107, 115, 123, 131, 138, 146, 154,
        161, 169, 176, 184, 192, 200, 207, 215, 223, 231, 238,
    ],
    [
        0, 8, 16, 25, 33, 41, 49, 58, 66, 74, 82, 90, 99, 107, 115, 123, 132, 140, 148, 156, 165,
        173, 181, 189, 197, 206, 214, 222, 230, 239, 247, 255,
    ],
];
