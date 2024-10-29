# wfcue

Merge/Split WAV,FLAC files and create CUE sheet

## Examples

Merge all wav files in the current working directory and create CUE sheet:

`wfcue merge --cue --title "Album" --performer "Artist" --rem "COMMENT wfcue" --verify --input *.wav --output "Artist - Album.wav"`

Merge selected wav files in the current working directory and create CUE sheet:

`wfcue merge --cue --title "Album" --performer "Artist" --rem "COMMENT wfcue" --verify --input 1.wav,2.wav,3.wav --output "Artist - Album.wav"`

Split a single large audio file containing the entire album into the separate audio tracks:

`wfcue split --input "Artist - Album.cue" --verify --format "%track%. %artist% - %title%"`

Split a single large audio file containing the entire album into the separate audio tracks and create multiple file CUE sheet:

`wfcue split --cue --input "Artist - Album.cue" --verify --format "%track%. %artist% - %title%"`

Merge all wav files in the current working directory and create CUE sheet also overwrite existing files and use silent mode:

`wfcue --force --silent merge --cue --title "Album" --performer "Artist" --rem "COMMENT wfcue" --rem "COMPOSER test" --verify --input *.wav --output "Artist - Album.wav"`
