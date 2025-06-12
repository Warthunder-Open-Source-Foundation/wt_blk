If further methods are desired, open an issue so that i can add them.


# Dev
Generate CI using  
```shell
 maturin generate-ci github -m python_bindings/Cargo.toml --platform linux windows macos > .github/workflows/pybindings_release.yml
```
Then remove any changes to name, deploy condition and targets unless desired.  