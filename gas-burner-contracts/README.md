## Setup steps

- deploy the owner-sc
- deploy gas-burner, giving it the owner-sc's address at init. Make sure it's payable by SC
- set gas-burner address in owner-sc
- change the owner of gas-burner to owner-sc
- unpause gas-burner through owner-sc
