import {
    Divider,
    DrawerBody,
    DrawerHeader,
    InlineDrawer,
    makeStyles,
    MenuItem,
    MenuList,
    Toolbar
} from '@fluentui/react-components';
import { DesktopToolboxFilled } from "@fluentui/react-icons"
import "./App.css";

const useStyles = makeStyles({
    root: {
        overflow: "hidden",
        display: "flex",
        height: "100%",
        width: "100%"
    },
    content: {
        flex: "1",
        padding: "16px",
        display: "flex",
        justifyContent: "center",
        alignItems: "flex-start",
        height: "100%",
        width: "100%",
    },
    contentPage: {
        flex: "1",
        padding: "16px",
        display: "flex",
        justifyContent: "center",
        alignItems: "flex-start",
        height: "100%",
        width: "100%",
    },
});

const App = () => {
    const styles = useStyles();
    return (
        <div className={styles.root}>
            <InlineDrawer size="small" open>
                <DrawerHeader className="p-0 m-0">
                    <Toolbar className="flex flex-row gap-x-6 items-center">
                        <DesktopToolboxFilled className="text-[30px]"/>
                        <h5 className="text-2xl font-bold inline">
                            Valet UI!
                        </h5>
                    </Toolbar>
                    <Divider/>
                </DrawerHeader>
                <DrawerBody>
                    <MenuList>
                        <MenuItem
                            icon={<DesktopToolboxFilled/>}
                            onClick={() => alert("Cut to clipboard")}
                        >
                            Dashboard
                        </MenuItem>
                        <MenuItem
                            icon={<DesktopToolboxFilled/>}
                            onClick={() => alert("Copied to clipboard")}
                        >
                            General
                        </MenuItem>
                        <MenuItem
                            icon={<DesktopToolboxFilled/>}
                            onClick={() => alert("Pasted from clipboard")}
                        >
                            Sites
                        </MenuItem>
                        <MenuItem
                            icon={<DesktopToolboxFilled/>}
                            onClick={() => alert("Pasted from clipboard")}
                        >
                            PHP
                        </MenuItem>
                        <MenuItem
                            icon={<DesktopToolboxFilled/>}
                            onClick={() => alert("Pasted from clipboard")}
                        >
                            Mail
                        </MenuItem>
                        <MenuItem
                            icon={<DesktopToolboxFilled/>}
                            onClick={() => alert("Pasted from clipboard")}
                        >
                            About
                        </MenuItem>
                    </MenuList>
                </DrawerBody>
            </InlineDrawer>
            <div>
                <Divider vertical className=" h-full"/>
            </div>
            <div className={styles.content}>
                <div className={styles.contentPage}>
                    <p>This is the page content</p>
                </div>
            </div>
        </div>
    );
}

export default App;
