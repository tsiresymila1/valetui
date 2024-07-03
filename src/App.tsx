import { makeStyles, InlineDrawer, DrawerHeader, DrawerBody,DrawerHeaderTitle } from '@fluentui/react-components';
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
        width: "100%"
    },
});

const App = () => {
    const styles = useStyles();
    return (
        <div className={styles.root}>
            <InlineDrawer size="small" open>
                <DrawerHeader>
                    <DrawerHeaderTitle>Always open</DrawerHeaderTitle>
                </DrawerHeader>
                <DrawerBody>
                    <p>Drawer content</p>
                </DrawerBody>
            </InlineDrawer>
            <div className={styles.content}>
                <p>This is the page content</p>
            </div>
        </div>
    );
}

export default App;
