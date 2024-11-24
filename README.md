# Chunked storage

I chose to implement a web-server that supports access to data with minimal latency using Rust and Axum framework.
Throughout the whole implementation I was testing the web-server by trying a MPEG-DASH live stream.

## Usage

To send data to the server you use the HTTP PUT method to an arbitrary path on the server. The data will be stored as a file under that path. If a file with such path exists it will be overwritten.
To get data from the server you use the HTTP GET method to a desired path on the server. The file will be sent as a response.
To delete data on the server you use the HTTP DELETE method to a desired path on the server. If such a file exists then it will be deleted.

## Implementation

The server uses a very simple in-memory file storage. Each PUT request is handled as an upload of a single file. It is then stored as a collection of chunks of data.
At the beginning of the put request, a new empty File is created, and chunks are continuously written to it. This enables GET requests to access the data even when the uploading is still in progress.
Files are stored as values in a HashMap where the keys are their paths.
When a get request for a file arrives, the server checks the HashMap for the path, and if it exists, it converts the File into a FileStream, which is sent as a response through Axum.
The FileStream checks for new data and sends them. Once the whole file has been read the stream ends.

## Troubles

Note: Some (maybe all) of the issues bellow might be my fault, but I did my best to find answers to them on GitHub pages of the crates and in the documentation.

One of the main troubles was figuring out how to send the "chunked" response. In the end, I decided to create a custom FileStream struct that implements the 'Stream' trait and therefore can be converted into Axum::Body which can be sent as a response.
Major issue was figuring out how the stream should decide when to end. It could not be when reaching the last chunk of the file as there could still be more chunks incoming. My first solution was to use a flag that said if a File was complete or not.
The stream could then check that flag and when reaching the last chunk it would know whether any more chunks are coming. This flag would be set at the end of the PUT request after all data would be polled.
That proved troublesome as I discovered that either Axum does not always properly detect the end of a chunked transfer or that ffmpeg does not correctly send the last zero-sized chunk to indicate the end of a chunked transfer.
This sometimes results in the PUT request stream getting stuck waiting for another chunk that is not coming. Timeout for Futures provided in 'tokio' crate does not work in this case because Axum internaly uses infinite loops in the stream implementation.
To resolve this issue, I added a timestamp to the File indicating when it was last updated. The FileStream checks that timestamp and if the time since then is over some threshold it considers the File as completed.

## Time spent

As this was my first time working with Axum, and Tokio as well, it probably took me longer than it should have. Overall I spent about 15-20 hours on this.
This included some deep dives into the documentation and even the source code as some bugs seemed rather strange to me. I also learned some things about the MPEG-DASH format and how it is used.

## LIST method

I did not implement this one as I ran into trouble figuring out how to use a custom HTTP method with Axum. However, I had a solution in my mind for this.
That would be using a prefix tree which would enable searching by each segment of the files path. For the final search among all the files with common prefix a HashMap could be used (just as it is used for all in this implementation).

## Efficiency

I am sure that this is not the most optimal solution possible, but it was quite "simple" to implement once I started to understand a little bit how to work with Axum and Streams.
A RwLock was used to lock files when reading or writing to them. The locks are always used for the minimal possible time, and thanks to the timeout in FileStream deadlocks should be avoided.

## Latency

Using the reference Dash player on the dash.js GitHub page reports a latency of the stream around 1s (which is as I understood the target latency set in the ffmpeg command).
