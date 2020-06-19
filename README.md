# vault-ordbok
Utility for injecting Vault values in k8s manifests and helm charts 

# Features
- Minimalistic CLI as much as possible
- Parsing, validating and injecting values from Vault in k8s manifests or helm charts 

# Supported commands
- `lookup` - get value by the given namespace and key 

# Syntax
Any executed command must be wrapped into the double curly braces with the specifying a command name as 
a regular function name. The used arguments for the command must be defined in the in parentheses.

Shortly speaking any command can be described in the `{{<command_name>(<args>)}}` format. The used command can 
contains multiple spaces between the curly braces, command name and its arguments.  

## Command examples
- `lookup` 

   This command accepts the namespace and key that needs to be extracted from this namespace. If specified less 
   than one or more than two arguments, the command will return a validation error.  
   Usage example:
   ```
      env:
        - name: SOME_SECRET
          value: {{ lookup ('/path/to/secret/', 'key') }}
   ```

# License
The vault-ordbok published under BSD license. For more details read [LICENSE](https://github.com/Relrin/vault-ordbok/blob/master/LICENSE) file.