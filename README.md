# tsp-toolkit-script-gen
The rust application to generate tsp scripts 

**Open Script** regenerates the script and waits until generation and the file
write succeed before opening the editor. **Save As** first asks for a destination
file, then generates the script directly into it and opens it once the write
succeeds. Generation or write failures are shown in the generator instead of
opening a file. Save As keeps the session's original output path; canceling the
dialog generates nothing.

Generation requires a configured session output path or an explicit Save As
destination. If neither is available, generation fails with an error; no default
output location is used.

Configuration edits and channel reallocation regenerate the script in the
background. Their data updates are returned immediately; if the background
generation or write fails, the error is shown in the generator afterwards.
