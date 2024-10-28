Easy config is a simple configuration language for apps which need some form of user configuration. Easy config strives to make it easy for programmers and other to configure a program with a configuration file. From the programmatic standpoint, the programmer needs only to slap a `derive(Config)` on the object representing the configuration. Then, easy config generates all the code needed for serializing and deserializing at compile time.

A configuration file might look something like
```
ip = 1.2.3.4
port = 2
content_folders = (
  src/content
  src/static
)
```

The programmer can then be sure that if they get a config object in their code, it has all the fields populated sensibly. 

The easy config language is very simple:
```
expr -> presence
      | pair
      | collection
presence -> TOKEN
pair -> presence "=" expr
collection -> "(" expr* ")" 
```
where a token is any string of characters that does not contain any whitespace, "\", "(", ")", "=", or "#". If a string must contain any of those characters it may be escaped by preceding it with a backslash. 

Because the `Config` trait is derived at compile time, there is no need to specify cardinality or types inside the config file. For instance, if a user has a struct like:
```rust
#[derive(Config)]
pub struct Top5 {
	members: [String; 5]
}
```

if a user attempts to supply the file:
```
members = (
	Wayne
	Sam
	John
)
```
They will get an error, telling them that the expected cardinality of `members` is 5.

# Comments
Easy config is designed to be shared between programmers and non programmers alike, and as such it is vital for programmers to be able to explain fields in the configuration file. While JSON doesn't allow the programmer to leave a comment, easy config does.
```rust
#[derive(Config)]
pub struct Top5 {
	#[EasyConfig(comment="The top 5 ice hockey players of all time.")]
	members: [String; 5]
}
```

When the `Top5` object is written to a string, it will come out as:
```
# The top 5 ice hockey players of all time.
members = (
	...
)
```

Comments may be placed anywhere in a easy config expression, with the understanding that a comment is everything from the "#" to the next newline.

# Why not serde?
Serde is a great piece of tooling, but has fundamentally different goals than easy config. Serde strives to serialize and deserialize as many types as possible as efficiently as possible. If this is your goal, then using serde is a much better idea. Easy config purposely makes it impossible to serialize most types. Config objects must not contain any references at all, and even the humble `&'static str` won't serialize. This is because easy config is designed to be loaded at program start, and contain the information necessary for the user to specify how a program functions. Limiting the kinds of serializable types helps users design config types that make sense.
