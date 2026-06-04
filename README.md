# WIP: satnogs-client-rs

This is an implementation of a client for the SatNOGS network. I had some
goals in mind when programming this:

- Minimalism: The client itself should do the bare necessities, to keep the codebase small and comprehensible
- Hackability: While the original satnogs client provides the pre- and post scripts already, I recycled that good idea with improvements:
    - Instead of calling scripts with some cmd-line options, we give them the whole metadata as json, so they can use anything the possibly can need
    - scripts are rum in lexicographical order from a scripts directory. You don't need to adjust a pre/post script
- Configurability: I wanted to simplify client configuration. This is achieved by two ways:
    - Comprehensible, overridable toml configuration format (instead of many env vars)
    - web-based configuration interface, for those who don't like command lines

Please consider, that this is first: A Work-In-Progress. There are not all features integrated, the original satnogs client has (and maybe never will be) and second: This client is intended for advanced computer users. I can not support you on setting up this, besides providing the code and explanations in this repository. If that doesn't help you at all, you should better go for the official satnogs client and its deployment [tutorials](https://community.libre.space/t/tutorial-how-to-install-satnogs-groundstation-for-newbie-pro-for-static-and-rotator/14200).

Besides that, if you are still here: Welcome, have fun experimenting and feel free to give me feedback in issues. :)
