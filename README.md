# rush
A toy rust shell (in rust newbie's coding style, maybe rewritten sometime later).

HW0 of ShanghaiTech IST course 2017s.

> ### Input language
>
> The shell reads commands from the standard input and execute them. We specify the input using Extended Backus-Naur Form (EBNF):
>
> ```
> Production  = production_name "=" [ Expression ] "." .
> Expression  = Alternative { "|" Alternative } .
> Alternative = Term { Term } .
> Term        = production_name | token [ "…" token ] | Group | Option | Repetition .
> Group       = "(" Expression ")" .
> Option      = "[" Expression "]" .
> Repetition  = "{" Expression "}" .
> ```
>
> Productions are expressions constructed from terms and the following operators, in increasing precedence:
>
> ```
> |   alternation
> ()  grouping
> []  option (0 or 1 times)
> {}  repetition (0 to n times)
> ```
>
> The input is encoded in Unicode. Each pair of adjacent tokens in the input are separated by one or more Unicode white space, except that the new line character need not be preceded or followed by white space. The following specification describes the input language.
>
> ```
> Input = { CommandLine } .
> CommandLine = [ Command [ "<" FileName ] { "|" Command } [ ">" FileName ] [ "&" ] ] new_line .
> Command = ( BuiltInCommand | ExecutableName ) { Argument } .
> ```
>
> ### Built-in commands
>
> - `cd` *directory*
>   Sets the current working directory to *directory*.
>
> - `exit`
>   Exits the shell.
>
> - `history`
>   Prints all the command lines that the user has entered in the chronological order. For each line
>
>   1. prints a counter that starts from 1, occupies 5 spaces, and is right-aligned;
>   2. prints two spaces;
>   3. prints the line;
>
>   For example, 
>
> ```
> 1  ls
> 2  ls | cat
> 3  cat < foo | cat | cat > bar
> 4  sleep 10 &
> ```
>
> - `jobs`
>   Prints the live command lines in the chronological order. For each command line,
>   prints its canonical representation as follows:
>
>   - Prints all the tokens: built-in commands, executables, arguments, file names for I/O redirection, `>`, `<`, and `|`. Do *not* print `&`.
>   - Separate every pair of adjacent tokens by one white space. Do not add white space at the beginning or end of the line.
>
>   Do *not* print the dead command lines, whose commands have all finished.
>
> - `kill` *pid*
>   Sends the signal `SIGTERM` to the process *pid*.
>
> - `pwd`
>   Prints the current working directory.
>
> If a line contains a single built-in command, the command executes in the current process and ignores I/O direction.
>
> ### External commands
>
> An external command is the name of an executable file. If the file name contains at least a slash (`/`), executes the file. Otherwise, searches each entry in the `PATH` environment variable in turn for the command. 
>
> External commands execute in child processes.
>
> ### I/O redirection
>
> - `<` *filename*
>
> Reads from *filename* instead of `stdio`.
>
> - `>` *filename*
>
> Writes to *filename* instead of `stdout`.
>
> ### Pipes
>
> ```
> command_1 | command_2 | ... | command_n
> ```
>
> Runs each command in a child process, where the standard output from `command_i` becomes the standard input into `command_{i+1}`. Optionally, the first command may redirect its input from a file, and the last command its output to a file.
>
> Each command may be either external or built-in.
>
> ### Background processes
>
> If a line has no trailing `&`, the shell waits for all the commands on this line to finish before reading the next line. Otherwise, the commands on the line run in the "background", and the shell reads the next line immediately.
>
> ### Error handling
>
> We will test your shell with only valid input. We encourage, but not require, your program to handle error input.
>
> ### Output
>
> The shell prints a prompt `$` followed by a space before reading each line of input.
