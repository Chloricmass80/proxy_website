# proxy_website

This project is a sneaky little backend that redirects immediately.
This was written for a Cybersecurity class, and all it does is track hits for my project.

This is not intended to be used maliciously.

## CLI Arguments:
* -h / --help: Display helpful information
* -r / --hits: Display the number of tracked hits
* no arguments serves the backend.

## Environment Variables:
Find environment variables in .env (or local.env) in the directory the binary is in.
If changing environment variables that are used to run the crate via cargo, edit in config/local.env.

* REDIRECT_URI: The URI to redirect to. (The executable will not check for URI validity so make sure it is correct or else it will throw an error or redirect to an invalid site.)
* TRACK_HITS: Weather or not to track hits. Simply don't declare this in local.env in order to turn this into a basic redirect backend that doesn't actually do anything other than redirect.
* BIND_ADDR: The port to bind to. Ex: To serve on http://localhost:3000, set this value to "0.0.0.0:3000"