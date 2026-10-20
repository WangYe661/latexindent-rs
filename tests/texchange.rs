//! texexchange 实战样例测试，覆盖 tests/cases/texchange/texexchange-test-cases.sh 部分直接调用。
mod common;
use common::{check, check_owned, report};

type PortedCase = (&'static str, &'static [&'static str], &'static str);

const PORTED_CASES: &[PortedCase] = &[
    (
        "112343-jon.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble", "-m"],
        "112343-jon-default.tex",
    ),
    (
        "112343-morbusg.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble,halignDelims"],
        "112343-morbusg-default.tex",
    ),
    (
        "112343-quinmars.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble,112343", "-m"],
        "112343-quinmars-default.tex",
    ),
    (
        "350144.tex",
        &[
            "-s",
            "-l",
            "indentPreamble,nextGroupPlot",
            "-o",
            "+-default-ngp",
            "-t",
        ],
        "350144-default-ngp.tex",
    ),
    (
        "350144.tex",
        &[
            "-s",
            "-l",
            "indentPreamble,nextGroupPlot-headings",
            "-o",
            "+-default-ngp-headings",
        ],
        "350144-default-ngp-headings.tex",
    ),
    (
        "39149-alain.tex",
        &[
            "-s",
            "-outputfile",
            "+-draw1",
            "-l",
            "indentPreamble,draw1",
            "-t",
        ],
        "39149-alain-draw1.tex",
    ),
    (
        "39149-loop-space.tex",
        &[
            "-s",
            "-outputfile",
            "+-no-add-named",
            "-l",
            "indentPreamble,no-add-named-braces",
        ],
        "39149-loop-space-no-add-named.tex",
    ),
    (
        "39149-loop-space2.tex",
        &[
            "-s",
            "-outputfile",
            "+-default",
            "-l",
            "indentPreamble,beamer",
        ],
        "39149-loop-space2-default.tex",
    ),
    (
        "39149-ruler-compass.sty",
        &["-s", "-outputfile", "+-default.sty", "-l", "beamer"],
        "39149-ruler-compass-default.sty",
    ),
    (
        "39149-morbusg.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "39149-morbusg-default.tex",
    ),
    (
        "39149-mark-wibrow1.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "39149-mark-wibrow1-default.tex",
    ),
    (
        "39149-mark-wibrow2.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "39149-mark-wibrow2-default.tex",
    ),
    (
        "39149-SztupY.tex",
        &[
            "-s",
            "-outputfile",
            "+-default",
            "-l=indentPreamble,fine-tuning-args1",
        ],
        "39149-SztupY-default.tex",
    ),
    (
        "103863-kiss-my-armpit1.tex",
        &["-s", "-o=+-default", "-l", "indentPreamble,pstricks"],
        "103863-kiss-my-armpit1-default.tex",
    ),
    (
        "135683-kiss-my-armpit1.tex",
        &["-s", "-o=+-default", "-l", "indentPreamble,pstricks"],
        "135683-kiss-my-armpit1-default.tex",
    ),
    (
        "348-cmhughes1.tex",
        &["-s", "-o=+-default", "-l=indentPreamble,pstricks"],
        "348-cmhughes1-default.tex",
    ),
    (
        "348-cmhughes2.tex",
        &["-s", "-o=+-default", "-l=indentPreamble"],
        "348-cmhughes2-default.tex",
    ),
    (
        "348-cmhughes2.tex",
        &[
            "-s",
            "-o=+-addplot-indent-rules",
            "-l",
            "addplot3,indentPreamble",
        ],
        "348-cmhughes2-addplot-indent-rules.tex",
    ),
    (
        "348-cmhughes1.tex",
        &["-s", "-o=+-mod5", "-m", "-l=348,indentPreamble,pstricks"],
        "348-cmhughes1-mod5.tex",
    ),
    (
        "43884.tex",
        &[
            "-s",
            "-o=+-default",
            "-l",
            "indentPreamble,round-brackets-args",
        ],
        "43884-default.tex",
    ),
    (
        "104498.tex",
        &["-s", "-o=+-default", "-l", "indentPreamble"],
        "104498-default.tex",
    ),
    (
        "353493.tex",
        &["-s", "-o=+-default", "-l", "indentPreamble"],
        "353493-default.tex",
    ),
    (
        "49814.tex",
        &[
            "-s",
            "-o=+-default",
            "-y=indentPreamble:1",
            "-l",
            "round-brackets-args",
        ],
        "49814-default.tex",
    ),
    (
        "49814-old.tex",
        &[
            "-s",
            "-o=+-default",
            "-y=indentPreamble:1",
            "-l=round-brackets-args",
        ],
        "49814-old-default.tex",
    ),
    (
        "571012.tex",
        &["-s", "-o=+-default", "-l=fine-tune"],
        "571012-default.tex",
    ),
    (
        "571012.tex",
        &["-s", "-o=+-mod1", "-l=draw-filldraw"],
        "571012-mod1.tex",
    ),
    ("578179.tex", &["-s", "-o=+-default"], "578179-default.tex"),
    (
        "578179.tex",
        &[
            "-s",
            "-o=+-mod1",
            "-y=defaultIndent:'  ';indentPreamble:1",
            "-l",
            "578179,nextGroupPlot",
        ],
        "578179-mod1.tex",
    ),
    (
        "36297-jake.tex",
        &[
            "-s",
            "-outputfile",
            "36297-jake-default",
            "-l",
            "indentPreamble",
        ],
        "36297-jake-default.tex",
    ),
    (
        "36297-patrick-hacker.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "36297-patrick-hacker-default.tex",
    ),
    (
        "52987-jake.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "52987-jake-default.tex",
    ),
    (
        "52987-anton.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "52987-anton-default.tex",
    ),
    (
        "46422-michi.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "46422-michi-default.tex",
    ),
    (
        "29293-christian-feuersanger.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "29293-christian-feuersanger-default.tex",
    ),
    (
        "29359-jake.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "29359-jake-default.tex",
    ),
    (
        "29359-marco-daniel.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "29359-marco-daniel-default.tex",
    ),
    (
        "29359-christian-feuersanger.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "29359-christian-feuersanger-default.tex",
    ),
    (
        "31276-peter-grill.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "31276-peter-grill-default.tex",
    ),
    (
        "11368-jake.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "11368-jake-default.tex",
    ),
    (
        "127375-jake.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "127375-jake-default.tex",
    ),
    (
        "127375-thomas.tex",
        &["-s", "-outputfile", "+-default", "-l", "indentPreamble"],
        "127375-thomas-default.tex",
    ),
    (
        "12207-christian-feuersanger.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "12207-christian-feuersanger-default.tex",
    ),
    (
        "12207-christian-feuersanger.tex",
        &[
            "-s",
            "-o",
            "+-pin-mlb1",
            "-m",
            "-l",
            "mlb-pin,indentPreamble",
        ],
        "12207-christian-feuersanger-pin-mlb1.tex",
    ),
    (
        "12207-jake.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "12207-jake-default.tex",
    ),
    (
        "155194-andrew-swan.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "155194-andrew-swan-default.tex",
    ),
    (
        "352549.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "352549-default.tex",
    ),
    (
        "351457-CarLaTeX.tex",
        &[
            "-s",
            "-o",
            "+-default",
            "-l",
            "indentPreamble,round-brackets-args",
        ],
        "351457-CarLaTeX-default.tex",
    ),
    (
        "351457-Andrew.tex",
        &[
            "-s",
            "-o",
            "+-default",
            "-l",
            "indentPreamble,preambleCommandsBeforeEnvironments",
        ],
        "351457-Andrew-default.tex",
    ),
    (
        "352502.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "352502-default.tex",
    ),
    (
        "352495.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "352495-default.tex",
    ),
    (
        "352396-Zarko1.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "352396-Zarko1-default.tex",
    ),
    (
        "354010.tex",
        &["-s", "-o=+-default", "-l", "indentPreamble"],
        "354010-default.tex",
    ),
    (
        "350642.tex",
        &["-s", "-o", "+-default"],
        "350642-default.tex",
    ),
    (
        "353035.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "353035-default.tex",
    ),
    (
        "253693.tex",
        &["-s", "-o", "+-default", "-l", "eq-bet-args", "-t"],
        "253693-default.tex",
    ),
    (
        "253693-Sean-Allred.tex",
        &["-s", "-o", "+-default", "-local=indentPreamble"],
        "253693-Sean-Allred-default.tex",
    ),
    (
        "253693-Sean-Allred.tex",
        &[
            "-s",
            "-m",
            "-o",
            "+-mod1",
            "-local=indentPreamble,253693,groupBeginEnd",
        ],
        "253693-Sean-Allred-mod1.tex",
    ),
    (
        "253693-John-Kormylo.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "253693-John-Kormylo-default.tex",
    ),
    (
        "253693-Manuel.tex",
        &["-s", "-o", "+-default", "-local", "indentPreamble"],
        "253693-Manuel-default.tex",
    ),
    (
        "96768-A-Ellett.tex",
        &["-s", "-o", "+-default"],
        "96768-A-Ellett-default.tex",
    ),
    (
        "96768-A-Ellett.tex",
        &["-s", "-o", "+-always-un-named", "-l", "always-un-named"],
        "96768-A-Ellett-always-un-named.tex",
    ),
    (
        "96768-egreg.tex",
        &["-s", "-o", "+-default"],
        "96768-egreg-default.tex",
    ),
    (
        "96768-egreg.tex",
        &["-s", "-o", "+-always-un-named", "-l", "always-un-named"],
        "96768-egreg-always-un-named.tex",
    ),
    (
        "56294-will-robertson.tex",
        &["-s", "-outputfile=+-default"],
        "56294-will-robertson-default.tex",
    ),
    (
        "56294-will-robertson.tex",
        &["-s", "-outputfile=+-groupBegEnd", "-l=groupBeginEnd"],
        "56294-will-robertson-groupBegEnd.tex",
    ),
    (
        "56294-Ahmed-Musa.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "56294-Ahmed-Musa-default.tex",
    ),
    (
        "56294-Ahmed-Musa.tex",
        &[
            "-s",
            "-o",
            "+-groupBegEnd",
            "-l",
            "indentPreamble,groupBeginEnd",
        ],
        "56294-Ahmed-Musa-groupBegEnd.tex",
    ),
    (
        "56294-Ahmed-Musa2.tex",
        &["-s", "-o", "+-default"],
        "56294-Ahmed-Musa2-default.tex",
    ),
    (
        "353559-james-fennell.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "353559-james-fennell-default.tex",
    ),
    (
        "353559-werner.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "353559-werner-default.tex",
    ),
    (
        "353559-egreg.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "353559-egreg-default.tex",
    ),
    (
        "353559-egreg-1.tex",
        &["-s", "-o", "+-default", "-l", "indentPreamble"],
        "353559-egreg-1-default.tex",
    ),
    (
        "158638-cmhughes.tex",
        &["-s", "-o=+-default", "-l", "indentPreamble"],
        "158638-cmhughes-default.tex",
    ),
    (
        "158638-cmhughes.tex",
        &["-s", "-o=+-items", "-l", "indentPreamble,158638-cmhughes"],
        "158638-cmhughes-items.tex",
    ),
    (
        "372580.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "372580-multicol.tex",
    ),
    (
        "371998.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "371998-multicol.tex",
    ),
    (
        "371998.tex",
        &[
            "-s",
            "-o=+-multicol5",
            "-l",
            "../alignment/multiColumnGrouping,../alignment/tabular5",
        ],
        "371998-multicol5.tex",
    ),
    (
        "371998.tex",
        &[
            "-s",
            "-o=+-multicol1",
            "-l",
            "../alignment/multiColumnGrouping1",
        ],
        "371998-multicol1.tex",
    ),
    (
        "371998.tex",
        &[
            "-s",
            "-o=+-multicol15",
            "-l",
            "../alignment/multiColumnGrouping1,../alignment/tabular5",
        ],
        "371998-multicol15.tex",
    ),
    (
        "371998-heiko.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "371998-heiko-multicol.tex",
    ),
    (
        "371319.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "371319-multicol.tex",
    ),
    (
        "369242.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "369242-multicol.tex",
    ),
    (
        "368176.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "368176-multicol.tex",
    ),
    (
        "367696.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "367696-multicol.tex",
    ),
    (
        "367278-min.tex",
        &["-s", "-o=+-mod1"],
        "367278-min-mod1.tex",
    ),
    (
        "367278.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "367278-multicol.tex",
    ),
    (
        "366841-zarko.tex",
        &["-s", "-o=+-out", "-l=longtabu"],
        "366841-zarko-out.tex",
    ),
    ("365928.tex", &["-s", "-o=+-default"], "365928-default.tex"),
    (
        "365928.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "365928-multicol.tex",
    ),
    (
        "365901.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "365901-multicol.tex",
    ),
    (
        "365620.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "365620-multicol.tex",
    ),
    (
        "364871.tex",
        &["-s", "-o=+-default", "-m"],
        "364871-default.tex",
    ),
    (
        "364871.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-m",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "364871-multicol.tex",
    ),
    (
        "364071.tex",
        &[
            "-s",
            "-o=+-default",
            "-l",
            "../filecontents/indentPreambleYes,longtabu",
        ],
        "364071-default.tex",
    ),
    (
        "364071.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "tabu,../filecontents/indentPreambleYes,longtabu",
        ],
        "364071-multicol.tex",
    ),
    ("359791.tex", &["-s", "-o=+-default"], "359791-default.tex"),
    (
        "359791.tex",
        &["-s", "-o=+-multicol", "-l", "longtable"],
        "359791-multicol.tex",
    ),
    (
        "359294.tex",
        &["-s", "-o=+-multicol", "-l", "tabularx"],
        "359294-multicol.tex",
    ),
    (
        "349480.tex",
        &["-s", "-o=+-default", "-l", "at-bet-args"],
        "349480-default.tex",
    ),
    (
        "349480.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping,at-bet-args",
        ],
        "349480-multicol.tex",
    ),
    ("348102.tex", &["-s", "-o=+-default"], "348102-default.tex"),
    (
        "348102.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "348102-multicol.tex",
    ),
    (
        "348102-mod.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "348102-mod-multicol.tex",
    ),
    ("347155.tex", &["-s", "-o=+-default"], "347155-default.tex"),
    (
        "347155.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "347155-multicol.tex",
    ),
    (
        "345211.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "345211-multicol.tex",
    ),
    ("342914.tex", &["-s", "-o=+-default"], "342914-default.tex"),
    (
        "342914.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "342914-multicol.tex",
    ),
    ("342697.tex", &["-s", "-o=+-default"], "342697-default.tex"),
    (
        "342697.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "342697-multicol.tex",
    ),
    (
        "342679.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "342679-multicol.tex",
    ),
    (
        "342288.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "342288-multicol.tex",
    ),
    (
        "340141.tex",
        &["-s", "-o=+-multicol", "-l", "tabularx"],
        "340141-multicol.tex",
    ),
    ("335633.tex", &["-s", "-o=+-default"], "335633-default.tex"),
    (
        "335633.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "335633-multicol.tex",
    ),
    (
        "333469.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "333469-multicol.tex",
    ),
    (
        "333469.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "333469-multicol.tex",
    ),
    (
        "331415.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "331415-multicol.tex",
    ),
    (
        "330385-min.tex",
        &["-s", "-o=+-mod1"],
        "330385-min-mod1.tex",
    ),
    (
        "330385.tex",
        &[
            "-s",
            "-o=+-multicol",
            "-l",
            "../alignment/multiColumnGrouping",
        ],
        "330385-multicol.tex",
    ),
    (
        "329346.tex",
        &["-s", "-o=+-multicol", "-l", "tabularstar", "-m"],
        "329346-multicol.tex",
    ),
    ("248166.tex", &["-s", "-o=+-out"], "248166-out.tex"),
    (
        "141087.tex",
        &["-s", "-o=+-out", "-l=pgfplotstableread"],
        "141087-out.tex",
    ),
    ("587491.tex", &["-s", "-o=+-mod1"], "587491-mod1.tex"),
    (
        "587491.tex",
        &["-s", "-o=+-mod2", "-l", "sizefeatures"],
        "587491-mod2.tex",
    ),
    (
        "627902.tex",
        &["-s", "-o=+-mod1", "-l", "627902"],
        "627902-mod1.tex",
    ),
    ("645096.tex", &["-s", "-o=+-default"], "645096-default.tex"),
    (
        "645096.tex",
        &["-s", "-l", "645096", "-o=+-mod1"],
        "645096-mod1.tex",
    ),
    (
        "pcc-pr.tex",
        &["-s", "-o=+-default", "-l", "indentPreamble"],
        "pcc-pr-default.tex",
    ),
    (
        "pcc-pr-presentation.tex",
        &[
            "-s",
            "-o=pcc-pr-presentation-default",
            "-l",
            "indentPreamble",
        ],
        "pcc-pr-presentation-default.tex",
    ),
    (
        "pcc-pr.tex",
        &[
            "-s",
            "-o=+-max-indentation",
            "-l",
            "indentPreamble,../environments/max-indentation1",
        ],
        "pcc-pr-max-indentation.tex",
    ),
    (
        "667013.tex",
        &["-m", "-s", "-l", "667013", "-o=+-mod1"],
        "667013-mod1.tex",
    ),
    (
        "709049.tex",
        &["-s", "-l", "709049", "-m", "-o=+-mod1"],
        "709049-mod1.tex",
    ),
    (
        "709049.tex",
        &["-s", "-l", "709049a", "-m", "-o=+-mod2"],
        "709049-mod2.tex",
    ),
    (
        "83663.tex",
        &["-s", "-rv", "-l", "83663", "-o=+-mod1"],
        "83663-mod1.tex",
    ),
    (
        "tex-fmt-issue-32.tex",
        &["-s", "-m", "-l", "tex-fmt-issue-32", "-o=+-mod1"],
        "tex-fmt-issue-32-mod1.tex",
    ),
    (
        "tex-fmt-issue-72.tex",
        &["-s", "-l", "tex-fmt-issue-72.yaml", "-o=+-mod1"],
        "tex-fmt-issue-72-mod1.tex",
    ),
    (
        "tex-fmt-issue-74.tex",
        &["-s", "-m", "-l", "tex-fmt-issue-74", "-o=+-mod1"],
        "tex-fmt-issue-74-mod1.tex",
    ),
    (
        "tex-fmt-issue-85.tex",
        &["-s", "-o=+-mod1"],
        "tex-fmt-issue-85-mod1.tex",
    ),
    (
        "tex-fmt-issue-88.tex",
        &["-s", "-r", "-l", "tex-fmt-issue-88.yaml", "-o=+-mod1"],
        "tex-fmt-issue-88-mod1.tex",
    ),
    (
        "tex-fmt-issue-107.tex",
        &["-s", "-l", "tex-fmt-issue-107.yaml", "-o=+-mod1"],
        "tex-fmt-issue-107-mod1.tex",
    ),
    (
        "tex-fmt-issue-114.tex",
        &["-s", "-o=+-mod1"],
        "tex-fmt-issue-114-mod1.tex",
    ),
    (
        "tex-fmt-issue-114a.tex",
        &["-s", "-o=+-mod1"],
        "tex-fmt-issue-114a-mod1.tex",
    ),
    (
        "tex-fmt-issue-120.tex",
        &["-s", "-l", "tex-fmt-issue-120.yaml", "-o=+-mod1"],
        "tex-fmt-issue-120-mod1.tex",
    ),
    (
        "tex-fmt-issue-121.tex",
        &["-s", "-o=+-mod1"],
        "tex-fmt-issue-121-mod1.tex",
    ),
    (
        "tex-fmt-issue-124.tex",
        &["-s", "-o=+-mod1"],
        "tex-fmt-issue-124-mod1.tex",
    ),
];

/// 剩余待修（2026-10-18：9 → 4；`348-cmhughes1-mod5` 与 `12207-...-pin-mlb1` 两处
/// 正则编译 panic 已修，`39149`/`52987`/`56294` 已转正）。见 doc/pending-work.md §5A.6。
/// 剩余待修（2026-10-18：5 → 1）。`353493`/`141087` 见 §5A.8；
/// `353559`/`39149` 靠「参数体循环照抄 Perl（body 优先 + 可回溯，撞 limit 时
/// 回退所有格版）」修好，见 §5A.9。
/// 已无待修用例：`253693` 随「ifElseFi body 加原子组」修好（§5A.10）。
const PENDING: &[&str] = &[];

fn run_ported_cases(pending: bool) {
    for &(input, args, expected) in PORTED_CASES {
        if PENDING.contains(&expected) == pending {
            check_owned(
                "texexchange",
                input,
                &args
                    .iter()
                    .map(|arg| (*arg).to_string())
                    .collect::<Vec<_>>(),
                expected,
            );
        }
    }
    report();
}

#[test]
fn texexchange_ported() {
    run_ported_cases(false);
}


#[test]
fn texexchange_default() {
    check(
        "texexchange",
        "31672-Werner.tex",
        &["-s", "-o=+-default"],
        "31672-Werner-default.tex",
    );
    check(
        "texexchange",
        "31672-s1l3n0.tex",
        &["-s", "-o=+-default"],
        "31672-s1l3n0-default.tex",
    );
    check(
        "texexchange",
        "31672-Bernard.tex",
        &["-s", "-o=+-default", "-l=indentPreamble"],
        "31672-Bernard-default.tex",
    );
    check(
        "texexchange",
        "74878.tex",
        &["-s", "-o=+-default"],
        "74878-default.tex",
    );
    check(
        "texexchange",
        "5461.tex",
        &["-s", "-o=+-default"],
        "5461-default.tex",
    );
}

#[test]
fn texexchange_2441() {
    check(
        "texexchange",
        "2441-Bernard.tex",
        &["-s", "-o=+-default"],
        "2441-Bernard-default.tex",
    );
}

#[test]
fn texexchange_indent_preamble() {
    check(
        "texexchange",
        "112343-Tom-Bombadil.tex",
        &["-s", "-l=112343.yaml", "-o=+-mod1"],
        "112343-Tom-Bombadil-mod1.tex",
    );
    check(
        "texexchange",
        "112343-dcmst.tex",
        &[
            "-s",
            "-o=+-default",
            "-l=indentPreamble,tcolorboxAlignDelims",
        ],
        "112343-dcmst-default.tex",
    );
    check(
        "texexchange",
        "112343-gekkostate.tex",
        &["-s", "-o=+-default", "-l=indentPreamble"],
        "112343-gekkostate-default.tex",
    );
    check(
        "texexchange",
        "112343-gonzalo.tex",
        &["-s", "-o=+-default", "-l=indentPreamble"],
        "112343-gonzalo-default.tex",
    );
    check(
        "texexchange",
        "350144.tex",
        &["-s", "-l=indentPreamble", "-o=+-default"],
        "350144-default.tex",
    );
}

#[test]
fn texexchange_39149() {
    check(
        "texexchange",
        "39149-alain.tex",
        &["-s", "-outputfile=+-default", "-l=indentPreamble"],
        "39149-alain-default.tex",
    );
    check(
        "texexchange",
        "39149-alain.tex",
        &["-s", "-outputfile=+-draw", "-l=indentPreamble,draw"],
        "39149-alain-draw.tex",
    );
    check(
        "texexchange",
        "39149-loop-space.tex",
        &["-s", "-outputfile=+-default", "-l=indentPreamble"],
        "39149-loop-space-default.tex",
    );
    check(
        "texexchange",
        "39149-jake.tex",
        &["-s", "-outputfile=+-default", "-l=indentPreamble"],
        "39149-jake-default.tex",
    );
}

// ---------------------------------------------------------------------------
// 日志断言（上游脚本 L25-27 / L35 / L91）
// ---------------------------------------------------------------------------

/// L35 / L91：两处 `egrep 'found: ' indent.log > *.txt`（已实测通过）。
#[test]
fn texexchange_logs() {
    common::check_log_lines(
        "texexchange",
        "39149-alain.tex",
        &[
            "-s",
            "-outputfile",
            "+-draw1",
            "-l",
            "indentPreamble,draw1",
            "-t",
        ],
        "found: ",
        "39149.txt",
    );
    common::check_log_lines(
        "texexchange",
        "253693.tex",
        &["-s", "-o", "+-default", "-l", "eq-bet-args", "-t"],
        "found: ",
        "253693.txt",
    );
}

/// L25-27：`cp indent.log nextgroupplot.txt` + 两处 `perl -p0i` 截取
/// （从首个 `WARN:  Obsolete` 到首个 `INFO:`）。
#[test]
fn texexchange_nextgroupplot_log() {
    let case = common::run(
        "texexchange",
        "350144.tex",
        &[
            "-s",
            "-l",
            "indentPreamble,nextGroupPlot",
            "-o",
            "+-default-ngp",
            "-t",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    );
    case.assert_success();
    case.assert_log_section(
        "indent.log",
        "WARN:  Obsolete",
        Some("INFO:"),
        "nextgroupplot.txt",
    );
}
