import React, { FunctionComponent } from "react";
import { Button, Card, Col, ListGroup, Row } from "react-bootstrap";
import { noop } from "lodash";

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

const FileList: FunctionComponent<Props> = (props) => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { onClick = noop, removeItem = noop } = props;
  const fileList = props.fileList ?? [];

  return fileList.length > 0 ? (
    <ListGroup as="ul">
      {fileList.map((item: File, index) => (
        <ListGroup.Item key={index} onClick={(event) => onClick(item, event)}>
          <Row xs={2}>
            <Col className="p-1" xs={10}>
              {item.fileName}
            </Col>
            <Col xs={2}>
              <Button
                className={"float-right"}
                size={"sm"}
                variant={"outline-danger"}
              >
                x
              </Button>
            </Col>
          </Row>
        </ListGroup.Item>
      ))}
    </ListGroup>
  ) : (
    <Card>
      <Card.Body>No Watched Files</Card.Body>
    </Card>
  );
};

export default FileList;
