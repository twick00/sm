import React, { FunctionComponent } from "react";
import { noop } from "lodash";
import { HTMLTable, ITreeNode, Text, Tree, TreeNode } from "@blueprintjs/core";
import { Col, Container, Row } from "react-grid-system";
import { Table } from "@blueprintjs/table";

const Button = ({ children }: any) => <div>{children}</div>;
const Card = ({ children }: any) => <div>{children}</div>;
const ListGroup = ({ children }: any) => <div>{children}</div>;

type File = {
  path: string;
  fileName: string;
};

interface OwnProps {
  fileList?: Array<File>;
  onClick?: (item: File, ...args: any[]) => void;
  removeItem?: (item: File, ...args: any[]) => void;
}

type Props = OwnProps;

interface FileTreeProps {}

const t: ITreeNode[] = [
  {
    id: 0,
    hasCaret: true,
    icon: "folder-close",
    label: "Folder 0",
  },
  {
    id: 0,
    hasCaret: true,
    icon: "folder-close",
    label: "Folder 0",
  },
  {
    id: 0,
    hasCaret: true,
    icon: "folder-close",
    label: "Folder 0",
  },
];

const FileTree: FunctionComponent<FileTreeProps> = (props) => {
  return (
    <Tree contents={t}>
      <TreeNode depth={1} path={[1, 2, 3]} id={"1"} label={"Test"}></TreeNode>
    </Tree>
  );
};

const FileList: FunctionComponent<Props> = (props) => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { onClick = noop, removeItem = noop } = props;
  const fileList = props.fileList ?? [];

  return (
    <Container fluid>
      <Row>
        <Col>
          <Text>Watched Files</Text>
          <FileTree></FileTree>
        </Col>
        <Col>TEST</Col>
      </Row>
    </Container>
  );
};

export default FileList;
