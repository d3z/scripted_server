# Scripted Server
## A scriptable HTTP server for client testing

This project is an 'itch scratcher'. Some time ago, I ran into an issue with a misbehaving server that would regularly throw a 404 for the same request. It appeared to happen every 3 or 4 requests, or after a request to another path (I can't remember the exact details). I had no control over the server, but my client application was not handling the situation very well.

Because I couldn't change the server, it made sense for me to write a mock server that basically mimicked the bad behaviour and throw a 404 after the 3 request.

From this, I had idea for a more generalised, scriptable server that I could used to setup some server-side scenario like the one above in order to test that client applications handled them correctly.

Imagine you have an application that makes a request for some resource then uses some property of that resource to make another call to the service for a different resource. What happens if, for whatever reason, the server responds with a 404 on the second request, but only sometimes?

### Using the scripted server
Write a script file (more on that below) and run `scripted_server script_file` where `script_file` is the path to the script file on your computer.

### The script file
A script file specifies the requests and responses the server will make in order that it expects the client to call them. The server will follow these steps, matching requests from clients with the current step and returning the configured response, advancing to the next step and waiting to the next request from the client.

Here is an example script file with an explanation of how the server will use it.

```yaml
---
name: My first script file # used for logging
repeat: true # should the server loop over the steps or end, defaults to false
path: /api/contact # the path that all steps match on by default
steps:
    - name: OK # used for logging
      code: 200 # HTTP response code returned by this step
      content: "ok" # body of reponse to return (can also be path to a file)
      content-type: "text/plain" # defaults to "text/plain"
      times: 2 # how many times to use this step, defaults to 1
    - name: Error # used for logging
      path: /api/contact/123 # overrides the global path
      code: 404
```

