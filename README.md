# yourdle

[![Deploy to Fastly](https://deploy.edgecompute.app/button)](https://deploy.edgecompute.app/deploy)

Make your own word game, using Fastly's Compute@Edge 🦀

**To learn more about Compute@Edge, head over to the [Fastly Developer Hub](https://developer.fastly.com/learning/compute/)**.

## The code

The game logic is implemented in a single Compute@Edge service written in Rust 🦀 and compiled to WebAssembly, with daily word challenges retrieved from a Fastly KV Store.

There are no backends, no databases, and no infrastructure to worry about. 
