import React from "react";
import ReactDOM from "react-dom/client";
import { FluentProvider, webDarkTheme } from '@fluentui/react-components';
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <FluentProvider
            style={{width: "100%"}}
            theme={webDarkTheme}
        >
            <App/>
        </FluentProvider>
    </React.StrictMode>,
);
