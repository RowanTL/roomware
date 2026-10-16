module Main exposing (main)

import Browser
import Dict
import Html exposing (Html, button, div, form, input, p, text)
import Html.Attributes exposing (attribute, autofocus, disabled, placeholder, style, type_, value)
import Html.Events exposing (onClick, onInput, onSubmit)
import Http


main : Program () Model Msg
main =
    Browser.document
        { init = \_ -> ( LoggingIn (login Nothing), Cmd.none )
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }



-- MODEL


{-| The session cookie is HTTP only, so the page can't know it is logged in until the server says so.
-}
type Model
    = LoggingIn Login
    | LoggedIn Playback


type alias Login =
    { passcode : String
    , submitting : Bool
    , problem : Maybe LoginProblem
    }


type LoginProblem
    = WrongPasscode
    | LockedOut (Maybe Int)
    | SessionExpired
    | LoginFailed Http.Error


type Playback
    = Idle
    | Sending
    | Played
    | PlayFailed Http.Error


login : Maybe LoginProblem -> Login
login problem =
    { passcode = "", submitting = False, problem = problem }



-- UPDATE


type Msg
    = PasscodeChanged String
    | LoginSubmitted
    | GotLogin (Result LoginProblem ())
    | PlayPressed
    | GotPlay (Result Http.Error ())


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model ) of
        ( PasscodeChanged passcode, LoggingIn fields ) ->
            ( LoggingIn { fields | passcode = passcode }, Cmd.none )

        ( LoginSubmitted, LoggingIn fields ) ->
            if fields.submitting then
                ( model, Cmd.none )

            else
                ( LoggingIn { fields | submitting = True, problem = Nothing }, logIn fields.passcode )

        ( GotLogin (Ok ()), LoggingIn _ ) ->
            ( LoggedIn Idle, Cmd.none )

        ( GotLogin (Err problem), LoggingIn _ ) ->
            ( LoggingIn (login (Just problem)), Cmd.none )

        ( PlayPressed, LoggedIn playback ) ->
            if playback == Sending then
                ( model, Cmd.none )

            else
                ( LoggedIn Sending, playVideo )

        ( GotPlay (Ok ()), LoggedIn _ ) ->
            ( LoggedIn Played, Cmd.none )

        ( GotPlay (Err (Http.BadStatus 401)), LoggedIn _ ) ->
            ( LoggingIn (login (Just SessionExpired)), Cmd.none )

        ( GotPlay (Err error), LoggedIn _ ) ->
            ( LoggedIn (PlayFailed error), Cmd.none )

        _ ->
            ( model, Cmd.none )



-- HTTP


logIn : String -> Cmd Msg
logIn passcode =
    Http.post
        { url = "/login"
        , body = Http.stringBody "text/plain" passcode
        , expect = Http.expectStringResponse GotLogin loginResult
        }


{-| A string response rather than `expectWhatever`, since a lockout's wait is only in its headers.
-}
loginResult : Http.Response String -> Result LoginProblem ()
loginResult response =
    case response of
        Http.GoodStatus_ _ _ ->
            Ok ()

        Http.BadStatus_ metadata _ ->
            case metadata.statusCode of
                401 ->
                    Err WrongPasscode

                429 ->
                    Err (LockedOut (Dict.get "retry-after" metadata.headers |> Maybe.andThen String.toInt))

                code ->
                    Err (LoginFailed (Http.BadStatus code))

        Http.BadUrl_ url ->
            Err (LoginFailed (Http.BadUrl url))

        Http.Timeout_ ->
            Err (LoginFailed Http.Timeout)

        Http.NetworkError_ ->
            Err (LoginFailed Http.NetworkError)


playVideo : Cmd Msg
playVideo =
    Http.post
        { url = "/play-video"
        , body = Http.emptyBody
        , expect = Http.expectWhatever GotPlay
        }



-- VIEW


view : Model -> Browser.Document Msg
view model =
    { title = "RoomWare"
    , body =
        [ div
            [ style "display" "flex"
            , style "flex-direction" "column"
            , style "align-items" "center"
            , style "justify-content" "center"
            , style "min-height" "100vh"
            , style "margin" "0"
            , style "font-family" "sans-serif"
            , style "font-size" "1.5em"
            ]
            (case model of
                LoggingIn fields ->
                    viewLogin fields

                LoggedIn playback ->
                    viewPlayback playback
            )
        ]
    }


viewLogin : Login -> List (Html Msg)
viewLogin fields =
    [ form [ onSubmit LoginSubmitted, style "display" "flex", style "gap" "0.5em" ]
        [ input
            [ type_ "password"
            , attribute "inputmode" "numeric"
            , attribute "autocomplete" "current-password"
            , placeholder "Passcode"
            , autofocus True
            , value fields.passcode
            , onInput PasscodeChanged
            , disabled fields.submitting
            , style "font-size" "inherit"
            ]
            []
        , button [ type_ "submit", disabled fields.submitting, style "font-size" "inherit" ]
            [ text "Log in" ]
        ]
    , p [] [ text (Maybe.withDefault "" (Maybe.map describeLoginProblem fields.problem)) ]
    ]


viewPlayback : Playback -> List (Html Msg)
viewPlayback playback =
    [ button
        [ onClick PlayPressed
        , disabled (playback == Sending)
        , style "font-size" "2em"
        , style "padding" "1em 2em"
        ]
        [ text "Play video" ]
    , p [] [ text (describePlayback playback) ]
    ]


describeLoginProblem : LoginProblem -> String
describeLoginProblem problem =
    case problem of
        WrongPasscode ->
            "Wrong passcode."

        LockedOut (Just seconds) ->
            "Too many wrong passcodes. Try again in " ++ minutes seconds ++ "."

        LockedOut Nothing ->
            "Too many wrong passcodes. Try again later."

        SessionExpired ->
            "Your session has expired, enter the passcode again."

        LoginFailed error ->
            describeError error


describePlayback : Playback -> String
describePlayback playback =
    case playback of
        Idle ->
            ""

        Sending ->
            "Starting the video…"

        Played ->
            "The video has started."

        PlayFailed error ->
            describeError error


describeError : Http.Error -> String
describeError error =
    case error of
        Http.BadUrl url ->
            "Bad address: " ++ url

        Http.Timeout ->
            "RoomWare took too long to answer."

        Http.NetworkError ->
            "Can't reach RoomWare. Is the game master computer on and on this network?"

        Http.BadStatus code ->
            "RoomWare answered with error " ++ String.fromInt code ++ "."

        Http.BadBody _ ->
            "RoomWare sent an answer this page doesn't understand."


minutes : Int -> String
minutes seconds =
    case ceiling (toFloat seconds / 60) of
        1 ->
            "1 minute"

        n ->
            String.fromInt n ++ " minutes"
