I was talking to Sanchen who is actually the Rust org organizerhe, I said we have 30 minutes to show this and the essence of the talk is that can you create a map of the codebase so that it is easier to find something which is relevant to you. Now this is a very different take from where the vision started. The vision of this one, this particular tool was not into just searching for things. What was the vision about? The vision was about you give me a requirement, I'll create the solution for you in one go


But that is an a very very long vision and to be honest impractical if that is the right word for it extremely impractical no

We created
- filepath-filename-interface-name-line-number-start-line-number-end = address in cozodb
    - we attached code block to in rocksdb
    - we attached metadata to in rocksdb
- we were able to generate
    - dependency graphs of interface-address x relationship x interface-address in json
        - LLMs cannot understand relationships between interfaces
    - we could search for a piece of code via datalog query in cozodb
        - we could never actually use search queries of parseltongue cozodb because we used grep


