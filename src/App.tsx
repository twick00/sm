import React, { useEffect, useState } from "react";
import * as dialog from "tauri/api/dialog";
import * as req from "tauri/api/tauri";
import { emit, listen } from 'tauri/api/event'
import { html } from "diff2html";
import ReactHtmlParser from "react-html-parser";
import FileList from "./components/FileList";
import { castArray, find, union } from "lodash";
import { fileNameFromPath } from "./utils/fileUtils";
import { Button, Navbar } from "@blueprintjs/core";
import Navigation from "./components/Navigation";
import { Container, Row, Col } from "react-grid-system";
import path from 'path';

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

  useEffect(() => {
    listen("refreshWatchedFileListResponse", (vals) => {
      console.log("Found refreshWatchedFileListResponse!: ", vals);
    })
  }, [])

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

  const formatFileDetails = (pathVal: string): FileDetails => {
    return {
      fileName: path.parse(pathVal).base,
      path: pathVal
    }
  }

  const buildAddButton = (options: dialog.OpenDialogOptions) => async () => {
    const newWatchedFiles = await dialog.open(options);
    let watchedFiles = union(state.watchedFiles, castArray(newWatchedFiles).map(formatFileDetails))

    console.log('Sending to `addWatchedFiles`')
    setState({
      ...state,
      watchedFiles
    })
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

  const refresh = () => {
    emit("refreshWatchedFileListRequest", "");
  }

  return (
    <>
      <Navigation addFilesButton={addFilesButton} />
      <FileList fileList={state.watchedFiles} />
      <Button onClick={refresh}>Refresh</Button>
    </>
  );
}

export default App;
