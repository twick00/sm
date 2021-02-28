import "bootstrap/dist/css/bootstrap.css";
import React, { useEffect, useState } from "react";
import * as dialog from "tauri/api/dialog";
import * as req from "tauri/api/tauri";
import {
  Button,
  Card,
  Col,
  Container,
  Nav,
  Navbar,
  Row,
} from "react-bootstrap";
import { html } from "diff2html";
import ReactHtmlParser from "react-html-parser";
import FileList from "./components/FileList";
import { castArray, find } from "lodash";
import { fileNameFromPath } from "./utils/fileUtils";

type FileDetails = {
  path: string;
  fileName: string;
};

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
    <Container
      fluid={true}
      className="App px-0"
      // style={{ minHeight: "100vh", padding: 0, margin: 0 }}
    >
      <Navbar bg="dark" variant="dark">
        <Navbar.Brand>Cud SM</Navbar.Brand>
        <Nav className="mr-auto">
          <Nav.Link>Some stuff</Nav.Link>
        </Nav>
        <Nav>
          <Nav.Item>
            <Button
              className={"mr-2"}
              variant={"outline-primary"}
              onClick={addFilesButton}
            >
              + Add Files
            </Button>
            <Button variant={"outline-primary"} onClick={addDirButton}>
              + Add Folder
            </Button>
          </Nav.Item>
        </Nav>
      </Navbar>
      <Container fluid className={"p-2 pt-3"}>
        <Row className={"m-0"}>
          <Col xs={6}>
            <FileList fileList={state.watchedFiles} onClick={clickedItem} />
          </Col>
          <Col xs={6}>
            <Card className={"h-100 overflow-auto"}>
              {state.selectedFile ? (
                <Card.Body>
                  <Card.Title>{state.selectedFile.fileName}</Card.Title>
                  <Card.Text>{state.selectedFile.path}</Card.Text>
                  {state.selectedFileDetails ? (
                    <Card.Text>{state.selectedFileDetails}</Card.Text>
                  ) : (
                    "Oops"
                  )}
                </Card.Body>
              ) : (
                <Card.Body>No Details To Show</Card.Body>
              )}
            </Card>
          </Col>
        </Row>
      </Container>
    </Container>
  );
}

export default App;
