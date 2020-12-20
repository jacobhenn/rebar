-- always run executable with +RTS -N
-- import System.Posix.Signals
{-# LANGUAGE MultiWayIf #-}
import Control.Concurrent.Async
import Control.Concurrent
import System.Process
import Control.Monad
import Data.Sequence
import System.IO
import Data.Time
import Data.List

--------------------------------------------------------------------------------
data Delta = Delta { ind :: Int, new :: String } deriving (Eq, Show)

--------------------------------------------------------------------------------
(!) :: Seq a -> Int -> a
(!) = index

--------------------------------------------------------------------------------
threadDelaySecs :: Int -> IO ()
threadDelaySecs i = threadDelay $ i * 10 ^ 6

--------------------------------------------------------------------------------
-- get the stdout of an sh command and strip any ending newlines
runShellCommand :: String -> IO String
runShellCommand cmd = (\x
  -> if | last x == '\n' -> init x
        | otherwise      -> x)
          <$> readProcess "sh" ["-c", cmd] ""

--------------------------------------------------------------------------------
getDate :: IO String
getDate = formatTime defaultTimeLocale "%A (%B) %Y-%m-%d %H:%M:%S %Z (UTC%z)"
            <$> getZonedTime

--------------------------------------------------------------------------------
getTags :: IO String
getTags = runShellCommand "format_tag_status"

--------------------------------------------------------------------------------
getCharge :: IO String
getCharge = runShellCommand "acpi -i\
                          \| head -n 1\
                          \| grep -P -o '(?<=, )\\d*(?=%)'\
                          \| gdbar -fg '#ebcb8b' -bg '#434c5e'"

--------------------------------------------------------------------------------
-- wakes up every second to get the date
dateThread :: Chan Delta -> IO ()
dateThread tx = forever $ do
    Delta 0 <$> getDate
      >>= writeChan tx
    threadDelaySecs 1

--------------------------------------------------------------------------------
-- wakes up whenever herbstclient catches a tag_switched hook and updates the
-- tag_status module of the bar
tagsThread :: Chan Delta -> IO ()
tagsThread tx = forever $ do
    runShellCommand "herbstclient -w -c 1 tag_changed"
    Delta 1 <$> getTags
      >>= writeChan tx

--------------------------------------------------------------------------------
-- wakes up every minute to check the status of the battery
chargeThread :: Chan Delta -> IO ()
chargeThread tx = forever $ do
    Delta 2 <$> getCharge
      >>= writeChan tx
    threadDelaySecs 60

--------------------------------------------------------------------------------
newBar :: IO (Seq String)
newBar = sequence $ fromList
           [ getDate
           , getTags
           , getCharge ]

--------------------------------------------------------------------------------
updateBar :: Seq String -> Delta -> Seq String
updateBar bar delta = update (ind delta) (new delta) bar

--------------------------------------------------------------------------------
printBar :: Seq String -> IO ()
printBar b = do
    putStrLn $ " ^fg(#d08770)"   ++ b!0
            ++ "   ^fg(#88c0d0)" ++ b!1
            ++ " ^pa(1800)"
            ++ " ^fg(#ebcb8b)"   ++ b!2
    hFlush stdout

--------------------------------------------------------------------------------
barLoop :: Chan Delta -> Seq String -> IO ()
barLoop rx bar = do
    printBar bar
    barLoop rx
      =<< updateBar bar
        <$> readChan rx

--------------------------------------------------------------------------------
main :: IO ()
main = do
    -- delete the named pipe before terminating when SIGINT is recieved
    -- const ()
    --   <$> installHandler sigINT
    --     (Catch $ runShellCommand "rm $XDG_RUNTIME_DIR/.rebar")
    --       Nothing
    rx <- newChan
    mapM_ async $ [dateThread, tagsThread, chargeThread] <*> return rx
    wait =<< (async $ barLoop rx =<< newBar)
