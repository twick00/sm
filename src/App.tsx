import React, { useEffect, useState } from "react";
import * as dialog from "tauri/api/dialog";
import * as req from "tauri/api/tauri";
import { html } from "diff2html";
import ReactHtmlParser from "react-html-parser";
import FileList from "./components/FileList";
import { castArray, find } from "lodash";
import { fileNameFromPath } from "./utils/fileUtils";
import { Button, Navbar } from "@blueprintjs/core";
import Navigation from "./components/Navigation";
import { Container, Row, Col } from "react-grid-system";

type FileDetails = {
  path: string;
  fileName: string;
};

const Card = ({ children }: any) => <div>{children}</div>;
function App() {
  const [state, setState] = useState<{
    watchedFiles: Array<FileDetails>;
    selectedFile?: FileDetails;
    selectedFileDetails?: any;
  }>({
    watchedFiles: [],
    selectedFile: undefined,
    selectedFileDetails: undefined,
  });

  const convertResult = (input: {
    data?: number[];
  }): { data?: string } & { [key in string]?: unknown } => {
    const { data, ...rest } = input;
    if (data) {
      return { ...rest, data: String.fromCharCode(...data) };
    }
    return rest;
  };

  useEffect(() => {
    console.log("watchedFiles changed");
    req
      .promisified({
        cmd: "addWatchedFiles",
        watchedFiles: state.watchedFiles.map((v) => v.path),
      })
      .then((data: any) => {
        console.log(
          castArray(data).map((v) => {
            let t = convertResult(data);
            return { ...v, data: t };
          })
        );
      })
      .catch((e) => {
        console.log("Error in watchedFiles useEffect: ", e);
      });
  }, [state.watchedFiles]);

  useEffect(() => {
    if (state.selectedFile) {
      req
        .promisified({ cmd: "selectFile", selectFile: state.selectedFile.path })
        .then((res: any) => {
          console.log("Event: selectFile");
          if (Array.isArray(res)) {
            const result = res.map((v: any) => convertResult(v as any))[1];
            // html(result.data)
            console.log(html(result.data as string));
            setState({
              ...state,
              selectedFileDetails: JSON.stringify(result, null, 2),
            });
            console.log(result);
            return result;
          }
          return res;
        })
        .catch((e) => {
          console.log("Error in selectedFile useEffect: ", e);
        });
    }
  }, [state.selectedFile]);

  const buildAddButton = (options: dialog.OpenDialogOptions) => async () => {
    const newWatchedPaths = await dialog.open(options);

    setState(({ watchedFiles }) => {
      const newDedupedWatchedPaths = castArray(newWatchedPaths)
        .filter((newPath) => {
          return !find(watchedFiles, ({ path }) => path === newPath);
        })
        .map((path) => {
          return {
            path,
            fileName: fileNameFromPath(path),
          };
        });
      const newFileList = [...watchedFiles, ...newDedupedWatchedPaths];
      return {
        watchedFiles: newFileList,
        selectedFile: state.selectedFile
          ? state.selectedFile
          : newFileList[0] ?? undefined,
      };
    });
  };

  const addFilesButton = buildAddButton({
    multiple: true,
  });

  const addDirButton = buildAddButton({
    directory: true,
  });

  const clickedItem = (file: FileDetails) => {
    setState((state) => {
      return {
        ...state,
        selectedFile: file,
      };
    });
  };

  return (
    <>
      <Navigation />
      <FileList />
    </>
  );
}

export default App;
