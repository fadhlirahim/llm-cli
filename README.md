# open_ai_cli

A simple rust cli app


## Pre-requisite

Rust 1.75

Setup in your OS environment `OPENAI_TOKEN`


## Run locally

```
cargo run
```

```
>
what's the linux command to get current folder size in human readable format?

⢼ Thinking...
Ah, I see you want to know the current folder size in a more human-friendly format. Well, allow me to introduce you to the magical command:
`du`! It stands for "Disk Usage" and it's basically the librarian of Linux, helping you organize and measure the size of folders.

To get the folder size in a human-readable format, simply open your terminal, navigate to the directory you're curious about, and run this command:

```
du -sh
```

```




## To modify and see whole json response:

Raw json response parsed:

```

// Assuming you have a response named `res` from the client.request call
let res_body = hyper::body::to_bytes(res.into_body()).await?;

// Convert the bytes to a String
let res_body_string = String::from_utf8(res_body.to_vec())
    .expect("response was not valid UTF-8");

// Print the entire response body
println!("Response Body: {}", res_body_string);
```

```
Response Body: {
  "id": "chatcmpl-8kPHnqrnxYxl3xNaEqZ8daNbt1KsD",
  "object": "chat.completion",
  "created": 1706070979,
  "model": "gpt-3.5-turbo-0613",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hey there! I'm just a humble AI assistant, so I don't have feelings, but thanks for asking! How can I assist you today?"
      },
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 47,
    "completion_tokens": 30,
    "total_tokens": 77
  },
  "system_fingerprint": null
}

```
