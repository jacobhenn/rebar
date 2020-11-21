## Rebar is currently in beta. It has practically zero configurability and is not fully optimized. If you are not comfortable with haskell, you should probably wait for version 1.0.0.

## About

Rebar is an asynchronous, reactive statusbar. That means that modules are updated if and only if they need to be updated.

## Configurability...?

This bar will be very configurable in the future, I promise! For now, all I can do for you is document how to change the source code to add something to the bar.

If you are not using herbstluftwm, you will need to change the `tagStatus` and `getTags` functions to display your workspace information. It is highly reccomended to use this statusbar with a window manager that has a feature which lets you passively listen for changes in workspace status, like herbstluft or bspwm. If you don't have this feature, you will have to poll for changes in workspace status on a fast timer, which takes up an unreasonable amount of CPU.

Most modules are written in this format:

```haskell
getBlank :: IO String
getBlank = runShellCommand "The output of this shell command will be displayed on the bar"

blankThread :: Chan Delta -> IO ()
blankThread tx = forever $ do
    {- optional-} runShellCommand "This module will wait for this shell command to exit before updating"
    Delta {- index where this module appeara in the bar -} <$> getBlank
      >>= writeChan tx
    {- optional -} threadDelaySecs {- number of seconds to wait before running again  -}

-- you will also want to add your module's name to these three lists:
newBar = sequence $ fromList 
           [ getStuff
           , getThings
           , getBlank ]
           
-- for this next one, you only have to add the index of your module
printBar b = do
    putStrLn $ b!0
            ++ b!1
            ++ b!2
           
main = do
    -- ...
   mapM_ async $ [stuffThread, thingsThread, blankThread] <*> return rx 
   -- ...
```
