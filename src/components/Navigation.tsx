import React, { FunctionComponent } from "react";
import { Alignment, Button, ButtonGroup, Navbar } from "@blueprintjs/core";
import { noop } from "lodash";

interface OwnProps {
  addFilesButton?: (...args: any) => void;
  addDirButton?: (...args: any) => void;
}

type Props = OwnProps;

const Navigation: FunctionComponent<Props> = (props) => {
  const { addFilesButton = noop, addDirButton = noop } = props;

  return (
    <Navbar>
      <Navbar.Group align={Alignment.LEFT}>
        <Navbar.Heading>Cud SM</Navbar.Heading>
        <Navbar.Heading>Some stuff</Navbar.Heading>
      </Navbar.Group>
      <Navbar.Group align={Alignment.RIGHT}>
        <ButtonGroup>
          {/*<Button minimal icon={"folder-open"} onClick={addDirButton}>*/}
          {/*  Add Folder*/}
          {/*</Button>*/}
          <Button minimal icon={"document"} onClick={addFilesButton}>
            Add File
          </Button>
        </ButtonGroup>
      </Navbar.Group>
    </Navbar>
  );
};

export default Navigation;
